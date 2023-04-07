use std::alloc::{dealloc, Layout};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

pub struct Stack<T> {
    top: AtomicPtr<StackNode<T>>,
    threads_in_pop: AtomicU32,
    to_be_delete: AtomicPtr<StackNode<T>>,
}

struct StackNode<T> {
    data: Option<T>,
    next: Option<NonNull<StackNode<T>>>,
}

impl<T> StackNode<T> {
    fn new(data: T) -> Self {
        Self {
            data: Some(data),
            next: None,
        }
    }
}

unsafe impl<T> Send for Stack<T> {}

unsafe impl<T> Sync for Stack<T> {}

impl<T> Stack<T> {
    pub fn new() -> Self {
        let top = AtomicPtr::new(std::ptr::null_mut());
        let threads_in_pop = AtomicU32::new(0);
        let to_be_delete = AtomicPtr::new(std::ptr::null_mut());
        Self {
            top,
            threads_in_pop,
            to_be_delete,
        }
    }

    pub fn push(&self, data: T) {
        let mut node = Box::leak(Box::new(StackNode::new(data)));
        loop {
            let top = self.top.load(Ordering::Acquire);
            (*node).next = NonNull::new(top);
            if let Ok(_) =
                self.top
                    .compare_exchange_weak(top, node, Ordering::SeqCst, Ordering::Relaxed)
            {
                break;
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        self.threads_in_pop.fetch_add(1, Ordering::Release);
        let mut old_top;
        loop {
            old_top = self.top.load(Ordering::Acquire);
            if !old_top.is_null() {
                let next = unsafe {
                    let ne = (*old_top).next;
                    if ne.is_some() {
                        ne.unwrap().as_ptr()
                    } else {
                        std::ptr::null_mut()
                    }
                };

                if let Ok(_) = self.top.compare_exchange_weak(
                    old_top,
                    next,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    break;
                }
            } else {
                break;
            }
        }
        let mut res = None;
        if !old_top.is_null() {
            res = unsafe { (*old_top).data.take() }
        }
        self.try_reclaim(old_top);
        res
    }

    fn try_reclaim(&self, old_top: *mut StackNode<T>) {
        let threads_in_pop = self.threads_in_pop.load(Ordering::Acquire);
        if threads_in_pop == 1 {
            let nodes_to_delete = self
                .to_be_delete
                .swap(std::ptr::null_mut(), Ordering::SeqCst);
            if self.threads_in_pop.fetch_sub(1, Ordering::SeqCst) - 1 == 0 {
                self.drop_nodes(nodes_to_delete);
            } else if !nodes_to_delete.is_null() {
                self.chain_pending_nodes(nodes_to_delete);
            }
            let layout = Layout::new::<StackNode<T>>();
            unsafe {
                std::alloc::dealloc(old_top as *mut u8, layout);
            }
        } else {
            self.threads_in_pop.fetch_sub(1, Ordering::Release);
            self.chain_pending_node(old_top);
        }
    }

    fn chain_pending_node(&self, node: *mut StackNode<T>) {
        self.link_to_be_delete(node, node);
    }

    fn chain_pending_nodes(&self, nodes: *mut StackNode<T>) {
        let mut last = nodes;
        unsafe {
            if !last.is_null() {
                loop {
                    let next = (*last).next;
                    if next.is_some() {
                        last = next.unwrap().as_ptr();
                    } else {
                        break;
                    }
                }
            }
        }
        self.link_to_be_delete(nodes, last);
    }

    fn link_to_be_delete(&self, first: *mut StackNode<T>, last: *mut StackNode<T>) {
        unsafe {
            loop {
                (*last).next = NonNull::new(self.to_be_delete.load(Ordering::Acquire));
                let next = if (*last).next.is_some() {
                    (*last).next.unwrap().as_ptr()
                } else {
                    std::ptr::null_mut()
                };
                if let Ok(_) = self.to_be_delete.compare_exchange_weak(
                    next,
                    first,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    break;
                }
            }
        }
    }

    fn drop_nodes(&self, list: *mut StackNode<T>) {
        unsafe {
            let mut nodes = list;
            if !nodes.is_null() {
                loop {
                    let next = (*nodes).next;
                    let _ = Box::from_raw(nodes);
                    if next.is_some() {
                        nodes = next.unwrap().as_ptr();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    pub fn traverse(&self)
    where
        T: Display,
    {
        let mut tmp = self.top.load(Ordering::Relaxed);
        unsafe {
            if !tmp.is_null() {
                loop {
                    let data = (*tmp).data.as_ref().unwrap();
                    println!("{}", data);
                    let next = (*tmp).next;
                    if next.is_some() {
                        tmp = next.unwrap().as_ptr();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        self.drop_nodes(self.to_be_delete.load(Ordering::Relaxed));
        let mut tmp = self.top.load(Ordering::Relaxed);
        if !tmp.is_null() {
            loop {
                let layout = Layout::new::<StackNode<T>>();
                let next = unsafe { (*tmp).next };
                unsafe {
                    dealloc(tmp as *mut u8, layout);
                }
                if next.is_some() {
                    tmp = next.unwrap().as_ptr();
                } else {
                    break;
                }
            }
        }
    }
}
