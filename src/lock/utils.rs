use std::cell::Cell;
use std::thread;
use std::time::Instant;



#[derive(Copy, Clone, PartialEq)]
pub(crate) enum State {
    PARK,
    RUNNING,
}

pub(crate) struct Node {
    pub(crate) next: *mut Node,
    pub(crate) thread: thread::Thread,
    pub(crate) state: Cell<State>,
}

impl Node {
    pub(crate) fn new() -> Self {
        Self {
            next: std::ptr::null_mut(),
            thread: thread::current(),
            state: Cell::new(State::RUNNING),
        }
    }
}

const STEP_LIMIT:u32=6;

pub(crate) struct Backoff{
    step:Cell<u32>
}

impl Backoff {
    pub(crate) fn new()->Self{
        Self{
            step:Cell::new(0)
        }
    }
    pub(crate) fn spin_light(&self){
        let step=self.step.get().min(STEP_LIMIT).pow(2);
        for _ in 0..step{
            std::hint::spin_loop();
        }
        self.step.set(self.step.get()+1)
    }
    pub(crate) fn spin_heavy(&self){
        let step=self.step.get().min(STEP_LIMIT).pow(2);
        if self.step.get()<=STEP_LIMIT {
            for _ in 0..step{
                std::hint::spin_loop();
            }
        }else{
            std::thread::yield_now();
        }
        self.step.set(self.step.get()+1)
    }
    pub(crate) fn is_complete(&self)->bool{
        self.step.get()>STEP_LIMIT
    }
}

