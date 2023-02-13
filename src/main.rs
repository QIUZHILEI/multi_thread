
struct Stu{
    name:String,
    age:u8
}

fn main() {
    let stu = Box::into_raw(Box::new(Stu{
        name:"qiu".to_string(),
        age:8
    }));
    {
        unsafe{Box::from_raw(stu)};
    }
    println!("stu is null? {}",stu.is_null());
}
