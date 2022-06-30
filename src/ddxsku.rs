use crate::common::httputils::get;
use crate::spider::{
    Novel, NovelID, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};
use anyhow::Result;
use scraper::Selector;
use sea_orm::DbConn;
use tokio::sync::mpsc::Receiver;

use static_init::dynamic;

const DATA_URL: &str = "http://www.ddxsku.com/";

const SELECT_SORT: &str = "div.main.m_menu > ul > li";
#[dynamic]
static SELECTOR_SORT: Selector = Selector::parse(SELECT_SORT).unwrap();

pub struct DDSpider<'a> {
    db: &'a DbConn,
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

impl Spider for DDSpider<'_> {
    async fn sorts(&self) -> Result<Vec<Sort>> {
        let page = get(DATA_URL).await?;
        for elem in page.select(&SELECTOR_SORT) {
            let href = {
                if let Some(link) = elem.value().attr("href") {
                    String::from(link)
                } else {
                    continue;
                }
            };
        }

        Ok()
    }
}
