use std::{sync::atomic::{AtomicPtr, AtomicUsize, Ordering}, cell::Cell};

#[derive(Debug,Default)]
struct IdGenerator{
    gen:AtomicUsize
}

impl IdGenerator{
    fn gen_id(&self)->usize{
        self.gen.fetch_add(1,Ordering::SeqCst)
    }
}

struct Node{
    next:*mut Node,
    thread:std::thread::Thread,
    thread_id:usize,
}

impl Node{
    fn new(id:usize)->Self{
        Self{
            next:std::ptr::null_mut(),
            thread:std::thread::current(),
            thread_id:id,
        }
    }
}

pub(crate) struct ReentrantLock {
    head:AtomicPtr<Node>,
    tail:AtomicPtr<Node>,
    hold_thread:Cell<usize>,
    id_generator:IdGenerator,
    fair:bool
}

impl ReentrantLock {
    pub(crate) fn new(f:Option<bool>)->Self{
        let fair=f.is_some();
        let id_generator=IdGenerator::default();
        let id=id_generator.gen_id();
        let node=Box::into_raw(Box::new(Node::new(id)));
        Self{
            head:AtomicPtr::new(node),
            tail:AtomicPtr::new(node),
            hold_thread:Cell::new(id),
            id_generator,
            fair
        }
    }

    pub(crate) fn lock(&self){

    }

    pub(crate) fn try_lock(&self){
        if self.fair{

        }else{
            
        }
    }

    pub(crate) fn unlock(&self){

    }


}


pub(crate) struct Condition{
    first:*mut Node,
    last:*mut Node
}

impl Condition{
    fn new(){

    }
    fn wait(){

    }
    fn signal(){

    }
    fn signal_all(){

    }
}