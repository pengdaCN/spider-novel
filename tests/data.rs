use std::env;
use chrono::format::Fixed::RFC3339;
use chrono::Utc;
use dotenv::dotenv;
use sea_query::Query;
use snowflake::SnowflakeIdGenerator;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::test;

#[test]
async fn insert_sort() {
    dotenv().ok();

    let mut gen = SnowflakeIdGenerator::new(1, 1);

    let link = env::var("DATABASE_URL").expect("MISS DATABASE_URL");

    let mut pool = SqlitePoolOptions::new().connect(&link).await.unwrap();

    let id = gen.generate();
    let created_at = Utc::now();
    sqlx::query!(r#"
    insert into sort
    (id, created_at, name, relation_spider_id, relation_id)
    values
    (?,?,?,?,?)"#, id, created_at, "x", "xx", 10)
        // .bind((gen.generate(), Utc::now(), "x", "xx", 10))
        .execute(&pool)
        .await
        .unwrap();
}

#[test]
async fn build_sql() {
}