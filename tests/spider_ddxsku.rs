use sea_orm::Database;
use spider_novel::common::snowid::set;
use spider_novel::ddxsku::{DDSpider, SortEntity};
use spider_novel::spider::{NovelID, Position, SortID, Spider};
use std::sync::Arc;
use tokio::fs::{DirBuilder, OpenOptions};
use tokio::io::AsyncWriteExt;
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

#[test]
async fn get_novels2() {
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

    let id: SortID = 6953709975051046913.into();

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

#[test]
async fn test_sections2() {
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

    let id: NovelID = 6953718276044230657.into();

    let mut tx = spider.sections_by_novel_id(&id, Position::Full).await.unwrap();
    let novel_name = "大唐全能奶爸";
    let path = &format!("/tmp/{novel_name}");
    DirBuilder::new().recursive(true).create(path).await.unwrap();

    loop {
        match tx.recv().await {
            Some(x) => {
                match x {
                    Ok(section) => {
                        let section_path = format!("{path}/{}-{}", section.seq, section.name);

                        let mut f = OpenOptions::new().create(true).write(true).open(&section_path).await.unwrap();
                        f.write_all(&section.text.as_bytes()).await.unwrap();

                        f.flush().await.unwrap();
                    }
                    Err(e) => {
                        println!("获取章节错误：{e}");
                    }
                }
            }
            None => break,
        }
    }
}