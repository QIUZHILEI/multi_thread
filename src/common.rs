use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, Ordering},
    thread::Thread,
    time::{Duration, UNIX_EPOCH, Instant},
};

const RUNNABLE: isize = 0;
const WAITING: isize = 1;
const COND_WAITING: isize = 2;

struct WaiterNode {
    next: *mut WaiterNode,
    status: RefCell<isize>,
    waiter: RefCell<Option<Thread>>,
    marker: PhantomData<Box<WaiterNode>>,
}

impl WaiterNode {
    fn new_raw() -> *mut Self {
        Box::into_raw(Box::new(Self {
            next: std::ptr::null_mut(),
            status: RefCell::new(RUNNABLE),
            waiter: RefCell::default(),
            marker: PhantomData::default(),
        }))
    }
    fn get_status(&self) -> isize {
        *self.status.borrow()
    }
    fn set_status(&self, s: isize) {
        self.status.replace(s);
    }
    fn unpark_waiter(&self) {
        self.waiter.take().unwrap().unpark();
    }
}

unsafe impl Sync for WaiterNode {}
unsafe impl Send for WaiterNode {}

trait Syncer {
    fn try_acquire(&self, arg: isize) -> bool;
    fn try_release(&self, arg: isize) -> bool;
    fn is_exclusively(&self) -> bool;
}

struct Locker {
    head: AtomicPtr<WaiterNode>,
    tail: AtomicPtr<WaiterNode>,
    sync: Box<dyn Syncer>,
}

impl Locker {
    fn new(sync: Box<dyn Syncer>) -> Self {
        let node = WaiterNode::new_raw();
        Self {
            head: AtomicPtr::new(node),
            tail: AtomicPtr::new(node),
            sync,
        }
    }

    fn enqueue(&self, node: *mut WaiterNode) {
        if !node.is_null() {
            loop {
                let t = self.tail.load(Ordering::Acquire);
                if let Ok(pre) =
                    self.tail
                        .compare_exchange(t, node, Ordering::SeqCst, Ordering::Relaxed)
                {
                    unsafe {
                        (*pre).next = node;
                    }
                    break;
                }
            }
        }
    }
    fn acquire_core(
        &self,
        mut node: *mut WaiterNode,
        arg: isize,
        timed: bool,
        time: u128,
        shared: bool,
    ) -> bool {
        let mut first;
        let mut spins: u8 = 0;
        let mut post_spins: u8 = 0;
        let mut pre = std::ptr::null_mut::<WaiterNode>();
        loop {
            first = self.head.load(Ordering::Relaxed).eq(&pre);
            //是队列里的第一个则可以执行抢锁操作，没有前驱代表不在队列，那就是刚来的线程，也可以执行抢锁操作
            if first || pre.is_null() {
                if self.sync.try_acquire(arg) {
                    //如果抢锁成成功，并且节点已经建立并在队列里，需要断开头节点的链接
                    if first {
                        let head = self.head.load(Ordering::Relaxed);
                        unsafe { (*head).next = std::ptr::null_mut() };
                        self.head.store(node, Ordering::Relaxed);
                        unsafe { (*node).waiter.replace(None) };
                        if shared {
                            self.signal_next(node);
                        }
                        //TODO!() head未清理，因为可能有其他线程持有head指针，直接清理是不安全的，因此需要借助GC算法
                    }
                    //不是第一个代表未入队那么清理这个节点是安全的
                    else{
                        if !node.is_null(){
                            unsafe{Box::from_raw(node)};
                        }
                    }
                    return true;
                }
            }
            //抢锁失败，应该建立节点准备入队
            if node.is_null() {
                node = WaiterNode::new_raw();
            }
            //节点建立好了，但是没有入队那就先入队
            else if pre.is_null() {
                unsafe { (*node).waiter.replace(Some(std::thread::current())) };
                let expect = self.tail.load(Ordering::Relaxed);
                if let Ok(t) = self.tail.compare_exchange_weak(
                    expect,
                    node,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    unsafe {
                        (*t).next = node;
                        //设置好node的前驱
                        pre = t;
                    }
                }
            } 
            //为了节点被唤醒时增加抢锁的成功率（因为此时可能有刚来的线程要抢锁，而本线程被唤醒）
            else if first && spins != 0 {
                spins -= 1;
                //on spin wait
            }
            //到这里说明node抢锁一定是失败的，因此为park做准备，应该先将node状态设置为WAITING
             else if unsafe { (*node).get_status() } == 0 {
                unsafe { (*node).set_status(WAITING) };
            } else {
                let nanos=time-std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
                spins = (post_spins << 1) | 1;
                post_spins = spins;
                if !timed {
                    std::thread::park();
                } else if nanos > 0 {
                    std::thread::park_timeout(Duration::from_nanos(nanos as u64));
                } else {
                    break;
                }
                unsafe { (*node).set_status(RUNNABLE) };
            }
        }
        false
    }

    // fn new_cond(&self)->CondVar{
    //     CondVar { first: (), last: () }
    // }
    fn signal_next(&self, node: *mut WaiterNode) {
        let next = unsafe { (*node).next };
        if !node.is_null() && !next.is_null() && unsafe { (*node).get_status() } != 0 {
            unsafe {
                (*node).set_status(0);
                (*node).unpark_waiter();
            }
        }
    }
    fn acquire(&self, arg: isize, shared: bool) {
        if !self.sync.try_acquire(arg) {
            self.acquire_core(std::ptr::null_mut(), arg, false, 0, shared);
        }
    }
    fn acquire_timeout(&self, arg: isize, time: u128, shared: bool) {
        if !self.sync.try_acquire(arg) {
            let now = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            self.acquire_core(std::ptr::null_mut(), arg, true, now + time, shared);
        }
    }
    fn release(&self, arg: isize) -> bool {
        if self.sync.try_release(arg) {
            self.signal_next(self.head.load(Ordering::Acquire));
            return true;
        }
        false
    }
    fn has_enqueued(&self) -> bool {
        let h = self.head.load(Ordering::Relaxed);
        unsafe { !(*h).next.is_null() }
    }
}

pub struct CondVar {
    first: *mut WaiterNode,
    last: *mut WaiterNode,
}

impl CondVar {}


struct Semaphore {
    permits: AtomicIsize,
    lock: Locker,
}

struct Reentrant {
    state: AtomicBool,
    lock: Locker,
}

struct CountDownLatch {
    count: AtomicIsize,
    lock: Locker,
}
