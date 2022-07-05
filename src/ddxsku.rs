use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::error;
use sea_orm::DbConn;
use skyscraper::html;
use skyscraper::xpath::parse::parse;
use skyscraper::xpath::Xpath;
use static_init::dynamic;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;

use crate::common::httputils::get;
use crate::ddxsku::data::{add_or_recover, sort_by_id};
use crate::spider::{
    Novel, NovelID, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};

pub mod data;

pub const DATA_URL: &str = "http://www.ddxsku.com/";

const SELECT_SORT: &str = r#"//div[@class="main m_menu"]/ul/li"#;
#[dynamic]
static SELECTOR_SORT: Xpath = parse(SELECT_SORT).unwrap();

const SELECT_PAGE: &str = r#"//div[@class="pagelink"]/a"#;
#[dynamic]
static SELECTOR_PAGE: Xpath = parse(SELECT_PAGE).unwrap();

pub struct DDSpider {
    db: Arc<DbConn>,
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

// 获取html中的属性
macro_rules! elem_attr {
    ($doc: expr, $elem:expr, attr=$name:expr, $or:ident) => {{
        use skyscraper::html::HtmlNode;
        if let Some(x) = $doc.get_html_node(&$elem).and_then(|x| match x {
            HtmlNode::Tag(inner) => inner.attributes.get($name),
            _ => None,
        }) {
            x.clone()
        } else {
            $or
        }
    }};
}

// 获取html中文本
macro_rules! elem_text {
    ($doc: expr, $elem: expr, $or:ident) => {{
        if let Some(x) = $elem.get_all_text(&$doc) {
            x
        } else {
            $or
        }
    }};
}

#[async_trait]
impl Spider for DDSpider {
    async fn sorts(&self) -> Result<Vec<Sort>> {
        let mut sorts = Vec::new();
        let raw_page = get(DATA_URL).await?;
        let page = html::parse(&raw_page)?;

        for elem in SELECTOR_SORT.apply(&page)? {
            let name: String = elem_text!(page, elem, continue);
            let link: String = elem_attr!(page, elem, attr = "href", continue);

            let id = add_or_recover(&self.db, &name, &link).await?;

            sorts.push(Sort {
                id: id.into(),
                name,
            })
        }

        Ok(sorts)
    }

    async fn novels_by_sort_id(
        self: Arc<Self>,
        id: &SortID,
        pos: Position,
    ) -> Receiver<Result<Novel>> {
        let (tx, rx) = channel(10);

        let id = id.clone();
        let handle = tokio::spawn(async move {
            let sort = sort_by_id(&self.db, &id).await?;
            let sort = if let Some(x) = sort {
                x
            } else {
                error!("未查询到分类");
                return Ok::<(), anyhow::Error>(());
            };

            let url = vec![DATA_URL, &sort.link].concat();
            let first_page = get(&url).await?;

            Ok::<(), anyhow::Error>(())
        });

        rx
    }

    async fn sections_by_novel_id(
        self: Arc<Self>,
        id: &NovelID,
        pos: Position,
    ) -> Receiver<Result<Section>> {
        todo!()
    }

    async fn search(&self, name: &str) -> Result<Option<Vec<Novel>>> {
        todo!()
    }
}
