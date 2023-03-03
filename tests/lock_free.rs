use std::fmt::{Display, Formatter};

#[derive(Debug)]
struct Stu {
    age: u8,
    name: String,
}

impl Display for Stu {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self.age.to_string() + "\t" + &self.name;
        f.write_str(&str)
    }
}

#[cfg(test)]
mod queue_test {
    use super::Stu;
    use multi_thread::lock_free::Queue;
    use std::{
        env,
        sync::{atomic::AtomicU32, Arc},
        thread::{self, JoinHandle},
        time::Duration,
    };

    //execute: cargo test --test lock_free -- queue_test::multi_thread all --nocapture
    //execute: cargo test --test lock_free -- queue_test::multi_thread enqueue --nocapture
    //execute: cargo test --test lock_free -- queue_test::multi_thread dequeue --nocapture
    #[test]
    fn multi_thread() {
        let args: Vec<String> = env::args().collect();
        if args.len() == 0 {
            panic!("Need a argument!(euqueue, dequeue or all)");
        }

        let arc = Arc::new(Queue::new());
        let mut joins: Vec<JoinHandle<()>> = Vec::new();
        let counter = Arc::new(AtomicU32::new(0));

        match args.get(2).unwrap() as &str {
            "dequeue" => {
                let queue = arc.clone();
                let init_size = 10000;
                println!("total:{init_size}");
                for index in 0..init_size {
                    queue.enqueue(Stu {
                        age: (index % 255 + 10) as u8,
                        name: index.to_string() + "stu",
                    })
                }
                dequeue(&arc, &mut joins, &counter);
            }
            "enqueue" => enqueue(&arc, &mut joins),
            "all" => {
                enqueue(&arc, &mut joins);
                dequeue(&arc, &mut joins, &counter);
            }
            _ => {
                panic!("Arguments is mistake!");
            }
        };

        for join in joins {
            join.join().unwrap();
        }

        let count = counter.clone().load(std::sync::atomic::Ordering::Relaxed);
        println!("dequeue: {count}");
        let queue = Arc::clone(&arc);
        let remains = queue.size();
        println!("remains: {remains}");
        println!("counter+remains={}", count + remains);
    }

    fn sleep(millis: i64) {
        let mut remains = millis;
        if remains < 0 {
            panic!("Sleep duration less than zero!");
        }
        while remains > 0 {
            let dur = Duration::from_micros(remains as u64);
            let now = std::time::Instant::now();
            thread::sleep(dur);
            let speed = now.elapsed().as_millis() as i64;
            remains -= speed;
        }
    }

    fn enqueue(arc: &Arc<Queue<Stu>>, joins: &mut Vec<JoinHandle<()>>) {
        let enqueue_threads = 100;
        let per_thread_enqueues: u32 = 10000;
        println!("total:{}", enqueue_threads * (per_thread_enqueues as i32));
        for i in 0..enqueue_threads {
            let name = "thread-".to_string() + &i.to_string();
            let queue = Arc::clone(&arc);
            let join = thread::Builder::new()
                .name(name.clone())
                .spawn(move || {
                    for index in 0..per_thread_enqueues {
                        let stu = Stu {
                            age: (index % 255) as u8,
                            name: name.clone(),
                        };
                        queue.enqueue(stu);
                    }
                })
                .unwrap();
            joins.push(join);
        }
    }

    fn dequeue(arc: &Arc<Queue<Stu>>, joins: &mut Vec<JoinHandle<()>>, counter: &Arc<AtomicU32>) {
        let dequeue_threads = 160;
        let per_thread_dequeues = 1000;
        for _ in 0..dequeue_threads {
            let queue = Arc::clone(&arc);
            let count = counter.clone();
            let join = thread::spawn(move || {
                for _ in 0..per_thread_dequeues {
                    if queue.dequeue().is_some() {
                        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            });
            joins.push(join);
        }
    }

    //execute: cargo test --test lock_free -- queue_test::single_thread --nocapture
    #[test]
    fn single_thread() {
        let queue = Queue::new();
        let enqueue_nums = 500;
        for index in 0..enqueue_nums {
            let age = (index % 255) as u8;
            queue.enqueue(Stu {
                age,
                name: age.to_string() + "stu",
            });
        }
        let remains = queue.size();
        println!("remains: {remains}");
        let dequeue_nums = 400;
        for _ in 0..dequeue_nums {
            queue.dequeue();
        }
        let remains = queue.size();
        println!("remains: {remains}");
    }
}

#[cfg(test)]
mod stack_test {
    // #[test]
    // fn stack_test() {
    //     let stack = Stack::new();
    //     let arc = Arc::new(stack);
    //     let mut joins = Vec::new();
    //     for i in 0..10 {
    //         let name = "thread-".to_string() + &i.to_string();
    //         let stack = Arc::clone(&arc);
    //         let join = thread::Builder::new()
    //             .name(name)
    //             .spawn(move || {
    //                 for index in 0..10 {
    //                     let stu = Stu {
    //                         age: 10 + index,
    //                         name: "qiu-".to_string() + &index.to_string(),
    //                     };
    //                     stack.push(stu);
    //                 }
    //             })
    //             .unwrap();
    //         joins.push(join);
    //     }
    //     thread::sleep(Duration::new(0, 100));
    //     for i in 10000..20000 {
    //         let name = "thread-".to_string() + &i.to_string();
    //         let stack = Arc::clone(&arc);
    //         let join = thread::Builder::new()
    //             .name(name)
    //             .spawn(move || {
    //                 for _ in 0..100 {
    //                     let stu: Option<Stu> = stack.pop();
    //                     if stu.is_some() {
    //                         println!("{:?}", stu);
    //                     } else {
    //                         println!("stu none");
    //                     }
    //                 }
    //             })
    //             .unwrap();
    //         joins.push(join);
    //     }

    //     for join in joins {
    //         join.join().unwrap();
    //     }
    //     let stack = Arc::clone(&arc);
    //     for index in 0..100 {
    //         let stu = Stu {
    //             age: 10 + index,
    //             name: "qiu-".to_string() + &index.to_string(),
    //         };
    //         stack.push(stu);
    //     }
    //     stack.traverse();
    // }
}
