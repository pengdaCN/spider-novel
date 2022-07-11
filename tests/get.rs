use skyscraper::html;
use spider_novel::common::doc;
use spider_novel::common::doc::WrapDocument;
use spider_novel::common::httputils::get;
use std::borrow::BorrowMut;
use tokio::test;

#[test]
async fn tget() {
    let x = get("http://www.ddxsku.com/").await.unwrap();
    let doc = doc::parse(&x).unwrap();
}

// #[test]
// async fn get_with_libxml() {
// let x = get("http://www.ddxsku.com/").await.unwrap();
// let parser = Parser::default_html();
//
// let doc = parser.parse_string(x).unwrap();
// }
