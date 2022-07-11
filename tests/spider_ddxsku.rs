use sea_orm::Database;
use spider_novel::common::snowid::set;
use spider_novel::ddxsku::DDSpider;
use spider_novel::spider::{NovelID, Position, SortID, Spider};
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

#[test]
async fn novels() {
    set(1, 1);
    let spider = ddxsku_spider().await;

    let xuanhuan: SortID = (6952249900922966018 as i64).into();
    let mut recv = spider
        .novels_by_sort_id(&xuanhuan, Position::Range(1..10))
        .await
        .unwrap();
    loop {
        match recv.recv().await {
            Some(x) => match x {
                Ok(x) => {
                    println!("{:?}", x);
                }
                Err(e) => {
                    println!("错误: {}", e);
                }
            },
            None => {
                break;
            }
        }
    }

    println!("ok")
}

#[test]
async fn sections() {
    set(1, 1);
    let spider = ddxsku_spider().await;
    let zheng_ya_zhu_tian: NovelID = (6952251514572378113 as i64).into();
    let mut recv = spider
        .sections_by_novel_id(&zheng_ya_zhu_tian, Position::Range(1..10))
        .await
        .unwrap();
    loop {
        match recv.recv().await {
            Some(x) => match x {
                Ok(x) => {
                    println!("{:?}", x);
                }
                Err(e) => {
                    println!("错误: {}", e);
                }
            },
            None => break,
        }
    }

    println!("ok")
}
