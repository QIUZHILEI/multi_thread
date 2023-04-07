use std::cell::Cell;
use std::sync::atomic::{AtomicIsize, AtomicPtr, Ordering};
use std::thread;
use std::time::{Instant};
use crate::lock::utils::{Node, State};


pub(crate) struct Semaphore {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
    permit: AtomicIsize,
    fair: bool,
}

impl Semaphore {
    pub(crate) fn new(permits: isize, fair: bool) -> Self {
        assert!(permits > 0);
        let node = Box::into_raw(Box::new(Node::new()));
        Self {
            head: AtomicPtr::new(node),
            tail: AtomicPtr::new(node),
            permit: AtomicIsize::new(permits),
            fair,
        }
    }
    pub(crate) fn acquire(&self, res: isize, deadline: Option<Instant>) -> bool {
        assert!(res > 0);
        let mut spins = 0;
        let mut post_spins = 0;
        let mut node = std::ptr::null_mut::<Node>();
        let mut pre_node = std::ptr::null_mut::<Node>();
        let mut first = false;
        loop {
            let tail_node = self.tail.load(Ordering::Acquire);
            if first || pre_node.is_null() {
                if self.try_acquire(res) {
                    if first {
                        unsafe { (*node).state.set(State::RUNNING) };
                        self.head.store(node, Ordering::Release);
                        // 原head泄露
                    } else if !node.is_null() {
                        drop(unsafe { Box::from_raw(node) });
                    }
                    self.signal_next(self.head.load(Ordering::Relaxed));
                    return true;
                }
            }

            if pre_node.is_null() {
                if node.is_null() {
                    node = Box::into_raw(Box::new(Node::new()));
                }
                if self.tail.compare_exchange(tail_node, node, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                    unsafe{(*tail_node).next=node};
                    pre_node = tail_node;
                }
            } else if post_spins != 0 {
                post_spins -= 1;
                std::hint::spin_loop();
            } else if unsafe { (*node).state.get() == State::RUNNING } {
                unsafe { (*node).state.set(State::PARK) };
            } else {
                spins += 1;
                post_spins = spins << 1;
                let (timed, end) = if deadline.is_some() {
                    (true, deadline.unwrap())
                } else {
                    (false, Instant::now())
                };
                let now = Instant::now();
                if !timed {
                    thread::park();
                } else if now < end {
                    thread::park_timeout(end - now);
                } else {
                    break;
                }
                unsafe { (*node).state.set(State::RUNNING) };
                first = true;
            }
        }
        false
    }

    fn signal_next(&self, h: *mut Node) {
        let first = unsafe { (*h).next };
        if !first.is_null() && unsafe { (*first).state.get() == State::PARK } {
            unsafe { (*first).thread.unpark() };
        }
    }

    pub(crate) fn release(&self, res: isize) {
        assert!(res > 0);
        loop {
            let current_permits = self.permit.load(Ordering::Acquire);
            if current_permits + res < current_permits {
                panic!("permit exceeds the maximum bound");
            }
            if self.permit.compare_exchange(current_permits, current_permits+res, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                break;
            }
        }
        self.signal_next(self.head.load(Ordering::Acquire));
    }
    pub(crate) fn is_fair(&self) -> bool {
        self.fair
    }
    pub(crate) fn try_acquire(&self, res: isize) -> bool {
        let current_permit = self.permit.load(Ordering::Relaxed);
        if current_permit - res > current_permit {
            panic!("permit exceeds the minimum bound");
        }
        if self.fair && !unsafe { (*self.head.load(Ordering::Acquire)).next.is_null() } {
            return false;
        }
        current_permit - res >= 0 && self.permit.compare_exchange(current_permit, current_permit - res, Ordering::SeqCst, Ordering::Relaxed).is_ok()
    }
    pub(crate) fn available_permits(&self) -> isize {
        self.permit.load(Ordering::Relaxed)
    }
}


#[cfg(test)]
mod test{
    use super::Semaphore;
    use std::{thread,sync::Arc, time::Duration};
    #[test]
    fn semaphore_fair() {
        let arc = Arc::new(Semaphore::new(16, false));
        let mut join_vec = Vec::new();
        let thread_num = 16;
        for i in 0..thread_num {
            let semaphore = arc.clone();
            let t_name = "thread-".to_owned() + &i.to_string();
            let join_handler = thread::Builder::new()
                .name(t_name)
                .spawn(move || {
                    semaphore.acquire(4, None);
                    println!("{} is working!", thread::current().name().unwrap());
                    thread::sleep(Duration::new(4, 0));
                    println!("{} complete!", thread::current().name().unwrap());
                    semaphore.release(4);
                })
                .unwrap();
            join_vec.push(join_handler);
        }
        join_vec.into_iter().for_each(|join_handler| {
            join_handler.join().unwrap();
        });
    }

    fn semaphore_non_fair(){

    }
    fn semaphore_timeout(){
        
    }
    fn semaphore_try_lock(){
        
    }
    #[test]
    fn semaphore_block(){

    }
}
