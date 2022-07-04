use std::panic;

fn main() {
    let result = panic::catch_unwind(|| {
        println!("Hello");
    });
    assert!(result.is_ok());

    let result = panic::catch_unwind(|| {
        panic!("oh, no");
    });

    assert!(result.is_err());

    println!("Hello, end of main()");
    panic!("Panic self!");
}