use rbatis::{CRUDTable, DateTimeUtc};
use serde::{Deserialize, Serialize};

#[derive(Debug, CRUDTable, Serialize, Deserialize)]
pub struct Sort {
    pub id: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
    pub name: String,
    pub relation_spider_id: Option<String>,
    pub relation_id: Option<i64>,
}