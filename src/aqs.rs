// use std::{sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, fence}, ptr::NonNull, thread::Thread, cell::RefCell};



// const RUNNING:i32=0;
// const WAITING:i32=1;
// const COND_WAITING:i32=2;
// const CANCELLED:i32=i32::MIN;


// trait Synchronizer{
//     fn try_acquire(&self,arg:i32)->bool;
//     fn try_release(&self,arg:i32)->bool;
// }

// pub struct SyncUtil{
//     head:*mut Node,
//     tail:*mut Node,
//     state:i32,
//     sync_impl:Box<dyn Synchronizer>
// }


// struct Node{
//     prev:*mut Node,
//     next:*mut Node,
//     waiter:*mut Thread,
//     status:AtomicI32
// }

// impl Node{
//     //原子操作连接节点
//     fn cpex_next(&self,c:*mut Node,v:*mut Node)->bool{
//         match cpex_node_cst(c, v){
//             Ok(_)=>true,
//             Err(_)=>false
//         }
//     }
    
//     fn cpex_prev(&self,c:*mut Node,v:*mut Node)->bool{
//         match cpex_node_cst(c, v){
//             Ok(_)=>true,
//             Err(_)=>false
//         }
//     }
    
//     fn get_unset_status(&self,v:i32)->i32{
//         let res = self.status.load(std::sync::atomic::Ordering::Relaxed) & !v;
//         self.status.store(res , std::sync::atomic::Ordering::Release);
//         res
//     }

//     fn set_prev_relaxed(p:*mut Node){

//     }

//     fn set_status_relaxed(s:i32){
        
//     }

//     fn clear_status(&self){
//         self.status.store(0, std::sync::atomic::Ordering::Release);
//     }
// }

// struct CondNode{

// }


// fn cpex_node_cst(c:*mut Node,v:*mut Node)->Result<*mut Node, *mut Node>{
//     let atomic = AtomicPtr::new(c);
//     atomic.compare_exchange(c, v, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::Relaxed)
// }

// fn store_node_relaxed(c:*mut Node,v:*mut Node)->Result<*mut Node, *mut Node>{
//     let atomic=AtomicPtr::new(c);
//     atomic.compare_exchange(c, v, std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed)
// }
