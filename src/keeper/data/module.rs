use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct Sort {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub name: String,
    pub relation_spider_id: Option<String>,
    pub relation_id: Option<i64>,
}