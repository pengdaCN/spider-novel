pub mod data;

use std::sync::Arc;
use crate::common::httputils::{get, WrapSend};
use crate::spider::{
    Novel, NovelID, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::error;
use scraper::Selector;
use sea_orm::DbConn;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::channel;

use crate::ddxsku::data::{add_or_recover, sort_by_id};
use static_init::dynamic;

pub const DATA_URL: &str = "http://www.ddxsku.com/";

const SELECT_SORT: &str = "div.main.m_menu > ul > li";
#[dynamic]
static SELECTOR_SORT: Selector = Selector::parse(SELECT_SORT).unwrap();

const SELECT_PAGE: &str = "div.pagelink > a";
#[dynamic]
static SELECTOR_PAGE: Selector = Selector::parse(SELECT_PAGE).unwrap();

pub struct DDSpider {
    db:  Arc<DbConn>,
}

impl SpiderMetadata for DDSpider {
    const SUPPORTED: Support = Support {
        get_sort: true,
        get_novel_from_sort: true,
        search_novel: true,
    };

    fn id() -> &'static str {
        DATA_URL
    }
}

macro_rules! elem_attr {
    ($elem:expr, attr=$name:expr, $or:ident) => {{
        let attr = if let Some(x) = $elem.value().attr($name) {
            String::from(x)
        } else {
            $or
        };

        attr
    }};
}

macro_rules! elem_text {
    ($elem: expr, $or:ident) => {{
        if let Some(text) = $elem.text().next() {
            String::from(text)
        } else {
            $or
        }
    }};
}

#[async_trait]
impl Spider for DDSpider {
    async fn sorts(&self) -> Result<Vec<Sort>> {
        let mut raw_links = Vec::new();
        {
            let page = get(DATA_URL).await.and_then(|x| {
                Ok(WrapSend::new(x))
            })?;
            for elem in page.select(&SELECTOR_SORT).map(|x| {
                WrapSend::new(x)
            }) {
                let name: String = elem_text!(elem, continue);
                let link: String = elem_attr!(elem, attr = "href", continue);


                raw_links.push((name, link))
            }
        }

        let mut sorts = Vec::with_capacity(raw_links.len());
        for (name, link) in raw_links {
            let id = add_or_recover(&self.db, &name, &link).await?;

            sorts.push(Sort {
                id: id.into(),
                name,
            })
        }

        Ok(sorts)
    }

    async fn novels_by_sort_id(self: Arc<Self>, id: &SortID, pos: Position) -> Receiver<Result<Novel>> {
        let (tx, rx) = channel(10);

        let id = id.clone();
        let handle = tokio::spawn(async move {
            let sort = sort_by_id(&self.db, &id).await?;
            let sort = if let Some(x) = sort {
                x
            } else {
                error!("未查询到分类");
                return Ok::<(), anyhow::Error>(())
            };

            let url = vec![DATA_URL, &sort.link].concat();
            let first_page = get(&url).await?;


            Ok::<(), anyhow::Error>(())

        });

        rx
    }

    async fn sections_by_novel_id(self: Arc<Self>, id: &NovelID, pos: Position) -> Receiver<Result<Section>> {
        todo!()
    }

    async fn search(&self, name: &str) -> Result<Option<Vec<Novel>>> {
        todo!()
    }
}

fn foo<T: Send> (x: T) -> T {x}