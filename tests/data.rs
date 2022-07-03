use sea_orm::Database;
use tokio::test;

use spider_novel::keeper::data::sort::{add_or_recover, list};
use spider_novel::spider::{Sort, SortID};

#[test]
async fn add() {
    let db = Database::connect("sqlite://data.db")
        .await
        .expect("连接数据库失败");

    let data: Vec<Sort> = vec![
        Sort {
            id: SortID::from(20 as i64),
            name: "xx2x3a".to_string(),
        },
        Sort {
            id: SortID::from(10 as i64),
            name: "xx123".to_string(),
        },
    ];

    add_or_recover(&db, "https://aa.com", &data)
        .await
        .expect("插入数据失败");
}

#[test]
async fn list_sort() {
    let db = Database::connect("sqlite://data.db")
        .await
        .expect("连接数据库失败");

    let data = list(&db, None).await.expect("获取数据失败");
    for x in data {
        println!("{:?}", x);
    }
}
