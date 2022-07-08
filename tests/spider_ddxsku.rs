use std::sync::Arc;
use sea_orm::Database;
use tokio::test;
use spider_novel::ddxsku::DDSpider;
use spider_novel::spider::Spider;

async fn ddxsku_spider() -> DDSpider {
    let db = Database::connect("sqlite://data.db")
        .await
        .expect("连接数据库失败");

    DDSpider::new(Arc::new(db))
}

#[test]
async fn sorts() {
    let spider = ddxsku_spider().await;

    for x in spider.sorts().await.unwrap() {
        println!("{:?}", x);
    }
}