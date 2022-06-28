use std::env;

use crate::keeper::data::sort;
use chrono::Duration;
use log::error;
use sea_orm::Database;

use crate::spider::{Sort, Spider, SpiderMetadata, Support};

pub mod data;

#[derive(Debug)]
pub struct Policy {
    pub sort_update_interval: Duration,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            sort_update_interval: Duration::days(7),
        }
    }
}

struct PropertySpider {
    id: &'static str,
    supported: &'static Support,
    inner: Box<dyn Spider>,
}

impl PropertySpider {
    fn new(id: &'static str, supported: &'static Support, inner: Box<dyn Spider>) -> Self {
        Self {
            id,
            supported,
            inner,
        }
    }
}

#[derive(Default)]
pub struct Keeper {
    spiders: Vec<PropertySpider>,
    policy: Policy,
}

impl Keeper {
    pub fn new() -> Self {
        Keeper::default()
    }

    pub fn add_spider<T>(&mut self, spider: T)
    where
        T: SpiderMetadata + Spider + 'static,
    {
        self.spiders.push(PropertySpider::new(
            T::id(),
            &T::SUPPORTED,
            Box::new(spider),
        ))
    }

    pub async fn run(&mut self) {
        let db = Database::connect(
            env::var("DATABASE_URL").expect("require DATABASE_URL environment variable"),
        )
        .await
        .expect("open database failed");

        loop {
            for x in (&self.spiders)
                .iter()
                .filter(|v| v.supported.get_sort == true && v.supported.get_novel_from_sort == true)
            {
                // 获取分类
                let sort = if let Ok(data) = sort::list(
                    &db,
                    Some(sort::ListOpt {
                        created_at_less_than: None,
                        relation_spider_id: Some(x.id),
                    }),
                )
                .await
                {
                    data.into_iter().map(Sort::from).collect()
                } else {
                    match x.inner.sorts().await {
                        Ok(sorts) => sorts,
                        Err(e) => {
                            error!("获取分类失败; id={}, err={}", x.id, e);
                            continue;
                        }
                    }
                };
            }
        }
    }
}
