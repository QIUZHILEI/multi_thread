use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::thread::Thread;
use std::time::Instant;
use crate::lock::utils::{Node, State};

pub struct CountDownLatch{
    head:AtomicPtr<Node>,
    tail:AtomicPtr<Node>,
    count:AtomicUsize
}

impl CountDownLatch{
    pub(crate) fn new(count:usize)->Self{
        assert!(count>0);
        let node=Box::into_raw(Box::new(Node::new()));
        Self{
            head:AtomicPtr::new(node),
            tail:AtomicPtr::new(node),
            count:AtomicUsize::new(count)
        }
    }
    pub(crate) fn count_down(&self){
        loop {
            let count=self.count.load(Ordering::Acquire);
            if count==0{
                break;
            }
            if self.count.compare_exchange(count,count-1,Ordering::SeqCst,Ordering::Relaxed).is_ok(){
                if count==1{
                    self.signal_all();
                }
                break;
            }
        }
    }

    fn signal_all(&self){
        let mut next=unsafe{(*self.head.load(Ordering::Relaxed)).next};
        loop {
            if next.is_null(){
                break;
            }
            unsafe{
                (*next).thread.unpark();
                next=(*next).next;
            };
        }
    }

    pub(crate) fn wait(&self,deadline:Option<Instant>){
        let node=Box::into_raw(Box::new(Node::new()));
        let mut enqueue=false;
        loop {
            let count=self.count.load(Ordering::Acquire);
            if count==0{
                if !enqueue{
                    drop(unsafe{Box::from_raw(node)});
                }
                break;
            }
            let tail=self.tail.load(Ordering::Acquire);
            if !enqueue{
                if self.tail.compare_exchange(tail,node,Ordering::SeqCst,Ordering::Relaxed).is_ok(){
                    unsafe{(*tail).next=node};
                    enqueue=true;
                }
            }else {
                let (timed,end)=if deadline.is_some(){
                    (true,deadline.unwrap())
                }else{
                    (false,Instant::now())
                };
                let now=Instant::now();
                if !timed{
                    std::thread::park();   
                }else if end>now{
                    std::thread::park_timeout(end-now);
                }else{
                    break;
                }
            }

        }
    }
    pub(crate) fn available_counts(&self)->usize{
        self.count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test{
    use super::CountDownLatch;
    use std::{sync::Arc, thread, time::Duration};

    #[test]
    fn countdown_normal(){
        let count_down=Arc::new(CountDownLatch::new(10));
        let mut threads=Vec::with_capacity(50);
        for _ in 0..10{
            let count=Arc::clone(&count_down);
            let t=thread::spawn(move ||{
                thread::sleep(Duration::from_secs(2));
                println!("count down");
                count.count_down();
            });
            threads.push(t);
        }
        for _ in 0..30{
            let count=Arc::clone(&count_down);
            let t=thread::spawn(move ||{
                println!("await");
                count.wait(None);
                println!("await complete!");
            });

            threads.push(t);
        }
        for ele in threads {
            ele.join().unwrap();
        }
    }

}