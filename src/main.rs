use std::thread;
use std::time::Duration;

struct Stu<'a> {
    name: &'a str,
    age: u8,
}


fn main() {
    let t1 = thread::spawn(|| {
       // thread::park_timeout(Duration::from_secs(4));
       //  println!("park complete!");
        thread::sleep(Duration::from_millis(20));
        println!("sleep complete!");
        thread::park_timeout(Duration::from_secs(3));
        println!("park complete!");
    });
    let t2 = thread::spawn(move || {
        t1.thread().unpark();
        t1.join().unwrap();
    });
    t2.join().unwrap();
}
