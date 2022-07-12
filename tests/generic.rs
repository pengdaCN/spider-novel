use std::collections::HashMap;

#[test]
fn parse_date() {}

#[test]
fn map() {
    struct X {
        i: i32,
    }

    let mut m = HashMap::new();
    m.insert(String::from("x"), X { i: 10 });

    m.get_mut("x").unwrap().i = 20;

    println!("{}", m.get_mut("x").unwrap().i);
}

#[test]
fn test_format() {
    let x = "fdasfaf_{}.html";
    println!("{}", format!(x, 10));
}
