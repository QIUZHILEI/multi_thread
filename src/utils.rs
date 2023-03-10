use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering},
    thread::{self, Thread},
};

use std::sync::atomic::AtomicPtr;

use crate::lock_free::Queue;

pub struct CountDownLatch {
    count: AtomicU32,
    waiter: RefCell<Option<Thread>>,
    has_waiter: AtomicBool,
}

unsafe impl Send for CountDownLatch {}

unsafe impl Sync for CountDownLatch {}

impl CountDownLatch {
    pub fn new(count: u32) -> Self {
        if count >= i32::MAX as u32 {
            println!("The count '{}' is too large!", count);
            std::process::abort();
        }
        Self {
            count: AtomicU32::new(count),
            waiter: RefCell::new(None),
            has_waiter: AtomicBool::new(false),
        }
    }

    pub fn wait(&self) {
        if let Ok(_) =
            self.has_waiter
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        {
            self.waiter.replace(Some(thread::current()));
            thread::park();
        }
    }

    pub fn countdown(&self) {
        loop {
            let tmp = self.count.load(Ordering::Acquire);
            if tmp == 0 {
                break;
            }
            if let Ok(pre) =
                self.count
                    .compare_exchange(tmp, tmp - 1, Ordering::SeqCst, Ordering::Relaxed)
            {
                if pre == 1 {
                    self.waiter.take().unwrap().unpark();
                }
                break;
            }
        }
    }

    pub fn get_count(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}

pub struct Semaphore {
    permits: AtomicI64,
    waiters_queue: Queue<Thread>,
}

unsafe impl Send for Semaphore {}

unsafe impl Sync for Semaphore {}

impl Semaphore {
    pub fn new(permits: i64) -> Self {
        if permits <= 0 {
            println!("The semaphore permits must be positive!");
            std::process::abort();
        }
        let permits = AtomicI64::new(permits);
        Self {
            permits,
            waiters_queue: Queue::new(),
        }
    }

    pub fn acquire(&self, acquires: i64) {
        if acquires <= 0 {
            panic!("Acquire permits must be positive!");
        }
        loop {
            let tmp_permits = self.permits.load(Ordering::Acquire);
            let after = tmp_permits - acquires;
            if after > tmp_permits {
                println!("Minimum permit count exceeded!");
                std::process::abort();
            }
            if let Ok(_) = self.permits.compare_exchange_weak(
                tmp_permits,
                after,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                if after < 0 {
                    self.waiters_queue.enqueue(thread::current());
                    thread::park();
                }
                break;
            }
        }
    }

    pub fn release(&self, releases: i64) {
        if releases <= 0 {
            panic!("Release permits must be positive!");
        }
        loop {
            let tmp_permits = self.permits.load(Ordering::Acquire);
            let after = tmp_permits + releases;
            if after < tmp_permits {
                println!("Maximum permit count exceeded!");
                std::process::abort();
            }
            if let Ok(pre) = self.permits.compare_exchange_weak(
                tmp_permits,
                after,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                if pre < 0 {
                    if let Some(t) = self.waiters_queue.dequeue() {
                        t.unpark();
                    };
                }
                break;
            }
        }
    }

    pub fn available(&self) -> i64 {
        self.permits.load(Ordering::Relaxed)
    }
}

pub struct ReentrantLock {
    waiter_queue: Queue<Thread>,
    lock: AtomicBool,
    hold_lock_thread: AtomicPtr<Thread>,
    fair: bool,
}

impl ReentrantLock {
    pub fn new(fair: bool) -> Self {
        Self {
            waiter_queue: Queue::new(),
            lock: AtomicBool::new(false),
            hold_lock_thread: AtomicPtr::new(std::ptr::null_mut()),
            fair,
        }
    }

    pub fn lock(&self) {
        let locked = if !self.fair {
            self.try_lock()
        } else {
            if !self.lock.load(Ordering::Acquire) && self.waiter_queue.is_empty() {
                self.try_lock()
            } else {
                false
            }
        };
        if !locked {
            self.waiter_queue.enqueue(thread::current());
            thread::park();
            self.hold_lock_thread
                .store(&mut thread::current() as *mut Thread, Ordering::Release);
        }
    }

    pub fn try_lock(&self) -> bool {
        match self
            .lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(_) => {
                self.hold_lock_thread
                    .store(&mut thread::current() as *mut Thread, Ordering::Release);
                true
            }
            Err(_) => false,
        }
    }

    pub fn unlock(&self) {
        let tid = thread::current().id();
        let hold_thread = self.hold_lock_thread.load(Ordering::Relaxed);
        if hold_thread.is_null() {
            println!("Illegal lock state!");
            std::process::abort();
        }
        let hold_thread_id = unsafe { (*hold_thread).id() };
        if tid != hold_thread_id {
            println!("Illegal lock state!");
            std::process::abort();
        }
        self.lock.store(false, Ordering::Release);
        if let Some(thread) = self.waiter_queue.dequeue() {
            thread.unpark();
        }
    }
}

unsafe impl Sync for ReentrantLock {}

unsafe impl Send for ReentrantLock {}
