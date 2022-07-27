use quick_js::Context;

#[test]
fn js_eval() {
    let context = Context::new().unwrap();

    let x = context.eval_as::<i32>("10 + 30").unwrap();
    println!("{x}")
}