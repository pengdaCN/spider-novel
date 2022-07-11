use sea_orm::Database;
use spider_novel::common::snowid::set;
use spider_novel::ddxsku::DDSpider;
use spider_novel::spider::Spider;
use std::sync::Arc;
use tokio::test;

async fn ddxsku_spider() -> DDSpider {
    let db = Database::connect("sqlite://data.db")
        .await
        .expect("连接数据库失败");

    DDSpider::new(Arc::new(db))
}

#[test]
async fn sorts() {
    set(1, 1);
    let spider = ddxsku_spider().await;

    for x in spider.sorts().await.unwrap() {
        println!("{:?}", x);
    }
}
