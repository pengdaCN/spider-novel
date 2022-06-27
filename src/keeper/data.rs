use anyhow::Result;

use crate::spider::{Sort, SortID};

pub mod module;

pub mod sort {
    use chrono::{DateTime, Utc};

    pub struct ListOpt<'a> {
        pub updated_at_less_than: Option<&'a DateTime<Utc>>,
        pub relation_spider_id: Option<&'a str>,
    }
}

pub trait SortRepo {
    fn add_or_recover(&mut self, id: &str, sorts: Vec<&Sort>) -> Result<()>;
    fn list(&self, id: &str, opts: sort::ListOpt<'_>) -> Result<Vec<SortID>>;
}