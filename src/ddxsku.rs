pub mod data;

use crate::common::httputils::get;
use crate::spider::{
    Novel, NovelID, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};
use anyhow::Result;
use scraper::Selector;
use sea_orm::DbConn;
use tokio::sync::mpsc::Receiver;
use async_trait::async_trait;

use static_init::dynamic;
use crate::keeper::data::sort::add_or_recover;

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
    ($elem:expr, $or:ident) => {
        {
            if let Some(text) = $elem.text().next() {
            String::from(text)
            } else {
                $or
            }
        }
    };
    ($elem:expr, attr=$name:expr, $or:ident) => {
        {
            let text = elem_attr!($elem, $or);
            let attr = if let Some(x) = $elem.value().attr($name) {
              String::from(x)
            } else {
                $or
            };

            (text, attr)
        }
    }
}

#[async_trait]
impl Spider for DDSpider<'_> {
    async fn sorts(&self) -> Result<Vec<Sort>> {
        let page = get(DATA_URL).await?;

        let mut sorts = Vec::new();
        for elem in page.select(&SELECTOR_SORT) {
            let (name, link): (String, String) = elem_attr!(elem, attr="href", continue);

            let id = data::add_or_recover(self.db, &name, &link).await?;
            sorts.push(Sort{
                id: id.into(),
                name
            })
        }

        Ok(sorts)
    }
}
