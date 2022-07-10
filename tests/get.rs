use skyscraper::html;
use spider_novel::common::doc;
use spider_novel::common::httputils::get;
use std::borrow::BorrowMut;
use tokio::test;
use spider_novel::common::doc::WrapDocument;

#[test]
async fn tget() {
    let x = get("http://www.ddxsku.com/").await.unwrap();
    let doc = doc::parse(&x).unwrap();
}

#[test]
async fn tget2() {
    let x = get("http://www.ddxsku.com/files/article/html/102/102212/39227055.html").await.unwrap();

    let doc = foo(WrapDocument::parse(&x));
    let contents = foo(doc.select("dd#contents"));

    println!("{}", contents.text());
    for x in contents.iter() {
    }
}

fn foo<T: Send + Sync>(x: T) -> T {x}

// #[test]
// async fn get_with_libxml() {
// let x = get("http://www.ddxsku.com/").await.unwrap();
// let parser = Parser::default_html();
//
// let doc = parser.parse_string(x).unwrap();
// }
