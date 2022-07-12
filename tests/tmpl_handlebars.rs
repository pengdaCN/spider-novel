// use handlebars::Handlebars;
use serde_json::json;
use tera::{Context, Tera};

// #[test]
// fn render_tmpl() {
//     let mut reg = Handlebars::new();
//     let _ = reg.register_template_string("hello", "hello {{name}}").unwrap();
//     let out = reg.render("hello", &json!({
//         "name": "xxx",
//     })).unwrap();
//
//     println!("{out}")
// }

#[test]
fn render_tera() {
    let mut tmpl = Tera::default();
    tmpl.add_raw_template("hello", "hello {{ name }}").unwrap();

    let data = Context::from_serialize(json!({
        "name": "pengda",
    }))
    .unwrap();

    let x = tmpl.render("hello", &data).unwrap();

    println!("{x}");
}
