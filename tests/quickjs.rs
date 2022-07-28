use quick_js::Context;

#[test]
fn js_eval() {
    let context = Context::new().unwrap();

    let x = context.eval_as::<i32>("10 + 30").unwrap();
    println!("{x}");

    context.set_global("age", 10).unwrap();

    let x = context.eval_as::<i32>("age + 1000").unwrap();
    println!("{x}");

    let x = context
        .eval_as::<String>(r#"JSON.stringify({name:"pengda"})"#)
        .unwrap();
    println!("{x}");
}
