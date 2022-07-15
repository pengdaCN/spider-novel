use sea_orm::Database;
use spider_novel::common::snowid::set;
use spider_novel::ddxsku::{DDSpider, SortEntity};
use spider_novel::spider::{NovelID, Position, SortID, Spider};
use std::sync::Arc;
use tokio::test;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

async fn ddxsku_spider() -> DDSpider {
    let db = Database::connect("sqlite://data.db")
        .await
        .expect("连接数据库失败");

    DDSpider::new(Arc::new(db))
}

#[test]
async fn set_sorts() {
    set(1, 1);
    let mut spider = ddxsku_spider().await;
    spider
        .set_sort(&vec![SortEntity {
            name: String::from("全部分类"),
            link: String::from(r#"http://www.ddxsku.com/top/lastupdate_{{page}}.html"#),
        }])
        .await
        .unwrap();
}

#[test]
async fn get_novels() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    set(1, 1);
    let mut spider = ddxsku_spider().await;
    spider.load_sorts().await.unwrap();

    let id: SortID = 6953631192986030081.into();

    let mut rx = spider.novels_by_sort_id(&id, Position::Full).await.unwrap();
    loop {
        match rx.recv().await {
            Some(x) => {
                println!("{:?}", x);
            }
            None => {
                break;
            }
        }
    }

    println!("ok")
}
