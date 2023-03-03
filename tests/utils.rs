use multi_thread::utils::*;
use rand::Rng;
use std::{sync::Arc, thread, time::Duration};

#[test]
fn countdown_test() {
    let count = 100;
    let test = move |threads| {
        let arc = Arc::new(CountDownLatch::new(count));
        let mut joins = Vec::new();
        for i in 0..threads {
            let count_down = arc.clone();
            let name = "thread-".to_string() + &i.to_string();
            let join_handler = thread::Builder::new()
                .name(name)
                .spawn(move || {
                    let current = thread::current();
                    let mut random = rand::thread_rng();
                    let dur = random.gen_range(1..5);
                    println!(
                        "{} is working! Sleeping {} seconds!",
                        current.name().unwrap(),
                        dur
                    );
                    thread::sleep(Duration::new(dur, 1000));
                    println!("{} is count down!", current.name().unwrap());
                    count_down.countdown();
                })
                .unwrap();
            joins.push(join_handler);
        }
        let count_down = arc.clone();
        count_down.wait();
        println!(
            "{} continue, current count is {}.",
            thread::current().name().unwrap(),
            count_down.get_count()
        );
        for handler in joins {
            handler.join().unwrap();
        }
    };
    //多于count的线程数 (其他线程未跑完，main就已经解除阻塞)
    let threads = 110;
    test(threads);
    println!("\n\n\n");
    //等于count的线程数 (其他线程正好跑完，main线程也正好解除阻塞)

    let threads = 100;
    test(threads);
    println!("\n\n\n");

    //小于count的线程 (count未减到0，会引发阻塞，解开注释导致测试线程阻塞不动)
    // let threads=50;
    // test(threads);
}

//cargo test --test utils -- semaphore_test --nocapture

#[test]
fn semaphore_test() {
    let arc = Arc::new(Semaphore::new(16));
    let mut join_vec = Vec::new();
    let thread_num = 16;
    for i in 0..thread_num {
        let semaphore = arc.clone();
        let t_name = "thread-".to_owned() + &i.to_string();
        let join_handler = thread::Builder::new()
            .name(t_name)
            .spawn(move || {
                semaphore.acquire(4);
                println!("{} is working!", thread::current().name().unwrap());
                thread::sleep(Duration::new(thread_num - i, 0));
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

#[test]
fn reentrant_lock_test() {
    let thread_nums = 1;
    let test = move |arc: Arc<ReentrantLock>| {
        let mut join_handlers = Vec::new();
        for i in 0..thread_nums {
            let lock = arc.clone();
            let name = "thread-".to_string() + &i.to_string();
            let join_handler = thread::Builder::new()
                .name(name.clone())
                .spawn(move || {
                    lock.lock();
                    println!("{} acquire lock ,and is working!", name.clone());
                    thread::sleep(Duration::new(3 - i, 0));
                    println!("{} release lock.", name.clone());
                    lock.unlock();
                })
                .unwrap();
            join_handlers.push(join_handler);
        }
        for handler in join_handlers {
            handler.join().unwrap();
        }
    };

    //非公平锁
    let non_fair_arc = Arc::new(ReentrantLock::new(false));
    test(non_fair_arc);
    //公平锁
    //let fire_arc = Arc::new(ReentrantLock::new(true));

    //重入性
    //let reentrant_arc = Arc::new(ReentrantLock::new(true));
}
