use chrono::Utc;
use fast_log::Config;
use rbatis::crud::CRUD;

use rbatis::rbatis::Rbatis;
use spider_novel::keeper::data::module::Sort;

#[test]
fn insert_sort() {
    fast_log::init(Config::new().console()).unwrap();

    tokio_test::block_on( async {
        let rb =Rbatis::new();
        rb.link("sqlite://data.db").await.unwrap();

        let sort = Sort{
            id: 100,
            created_at: Utc::now().into(),
            updated_at: None,
            name: "cc".to_string(),
            relation_spider_id: None,
            relation_id: None
        };

        rb.save(&sort, &[]).await.unwrap();
    })
}