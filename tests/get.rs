use skyscraper::html;
use tokio::test;
use spider_novel::common::httputils::get;
use nipper::Document;

#[test]
async fn tget() {
    let x = get("http://www.ddxsku.com/").await.unwrap();
println!("{x}");
    let _ = html::parse(&x).unwrap();
}

#[test]
async fn tget2() {
    let x = get("http://www.ddxsku.com/files/article/html/102/102212/39227055.html").await.unwrap();

    let doc = Document::from(&x);
    let x = doc.select("dd#contents").text();
    println!("{}", x);
}