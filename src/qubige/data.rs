pub mod novel;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Novel {
    pub id: i64,
    pub name: String,
    pub cover: String,
    pub author: String,
    pub last_updated_section: DateTime<Utc>,
    pub last_updated_section_at: DateTime<Utc>,
    pub last_updated_graped: DateTime<Utc>,
}