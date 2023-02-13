
fn stack_val()->*mut i32{
    let mut x=30;
    &mut x as *mut i32
}

// fn heap_val()->&String{
//     let s = String::from("hello");
//     &s
// }

#[test]
fn remove_test(){
    let res=stack_val();
    unsafe{
        println!("{}",*res);
    }
}