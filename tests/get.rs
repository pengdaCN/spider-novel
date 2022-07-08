use skyscraper::html;
use tokio::test;
use spider_novel::common::httputils::get;

#[test]
async fn tget() {
    let x = get("http://www.ddxsku.com/").await.unwrap();
println!("{x}");
    let _ = html::parse(&x).unwrap();
}