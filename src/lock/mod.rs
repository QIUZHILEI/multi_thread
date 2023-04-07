use std::ops::Deref;

mod reentrant;
mod semaphore;
mod utils;
mod countdown;

pub fn semaphore(){

}

pub fn reentrant_lock(){

}

pub fn count_down_latch(){

}

enum Lock{
    Semaphore(usize),
    ReentrantLock,
    CountDownLatch(usize)
}


struct Counter{
    inner:Lock
}

impl Counter{

}

impl Deref for Counter{
    type Target = Lock;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl Clone for Counter{
    fn clone(&self) -> Self {
        todo!()
    }
}


