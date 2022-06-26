pub mod data;

use std::pin::Pin;
use chrono::Duration;
use log::error;
use crate::spider::{Spider, SpiderMetadata, Support};

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
        where T: SpiderMetadata + Spider + 'static
    {
        self.spiders.push(PropertySpider::new(T::id(), &T::SUPPORTED, Box::new(spider)))
    }

    pub async fn run(&mut self) {
        loop {
            for x in (&self.spiders).iter().filter(|v| {
                v.supported.get_sort == true && v.supported.get_novel_from_sort == true
            }) {

                // 获取分类
                let sort = match x.inner.sorts().await {
                    Ok(sorts) => {
                        sorts
                    }
                    Err(e) => {
                        error!("获取分类失败; id={}, err={}", x.id, e);
                        continue;
                    }
                };
            }
        }
    }
}