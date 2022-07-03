pub mod data;

use crate::common::httputils::get;
use crate::spider::{
    Novel, NovelID, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};
use anyhow::Result;
use async_trait::async_trait;
use scraper::Selector;
use sea_orm::DbConn;
use tokio::sync::mpsc::Receiver;

use crate::ddxsku::data::add_or_recover;
use static_init::dynamic;

pub const DATA_URL: &str = "http://www.ddxsku.com/";

const SELECT_SORT: &str = "div.main.m_menu > ul > li";
#[dynamic]
static SELECTOR_SORT: Selector = Selector::parse(SELECT_SORT).unwrap();

pub struct DDSpider<'a> {
    db: &'a DbConn,
}

impl SpiderMetadata for DDSpider<'_> {
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
    ($elem:expr, $or:ident) => {{
        if let Some(text) = $elem.text().next() {
            String::from(text)
        } else {
            $or
        }
    }};
    ($elem:expr, attr=$name:expr, $or:ident) => {{
        let text = elem_attr!($elem, $or);
        let attr = if let Some(x) = $elem.value().attr($name) {
            String::from(x)
        } else {
            $or
        };

        (text, attr)
    }};
}

#[async_trait]
impl Spider for DDSpider<'_> {
    async fn sorts(&self) -> Result<Vec<Sort>> {
        let mut raw_links = Vec::new();
        {
            let page = get(DATA_URL).await?;
            for elem in page.select(&SELECTOR_SORT) {
                let (name, link): (String, String) = elem_attr!(elem, attr = "href", continue);

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
}
