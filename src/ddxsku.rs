use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{error, warn};
use sea_orm::DbConn;
use skyscraper::html;
use skyscraper::html::HtmlDocument;
use skyscraper::xpath::parse::parse;
use skyscraper::xpath::Xpath;
use static_init::dynamic;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;

use crate::common::httputils::get;
use crate::ddxsku::data::{add_or_recover, sort_by_id};
use crate::spider::{
    Novel, NovelID, NovelState, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};

pub mod data;

pub const DATA_URL: &str = "http://www.ddxsku.com/";

// 获取小说分类
const SELECT_SORT: &str = r#"//div[@class="main m_menu"]/ul/li"#;
#[dynamic]
static SELECTOR_SORT: Xpath = parse(SELECT_SORT).unwrap();

// 获取最后一条分页
const SELECT_LAST_PAGE: &str = r#"//a[@class="last"]"#;
#[dynamic]
static SELECTOR_LAST_PAGE: Xpath = parse(SELECT_LAST_PAGE).unwrap();

// 获取小说列表
const SELECT_NOVEL_TABLE: &str = r#"//tbody/tr"#;
#[dynamic]
static SELECTOR_NOVEL_TABLE: Xpath = parse(SELECT_NOVEL_TABLE).unwrap();

// 获取列表中的小说条目
const SELECT_NOVEL_ITEM: &str = r#"/td"#;
#[dynamic]
static SELECTOR_NOVEL_ITEM: Xpath = parse(SELECT_NOVEL_ITEM).unwrap();

// 获取小说封面链接
const SELECT_NOVEL_COVER: &str = r#"//div[@class="fl"][1]//img/@src"#;
#[dynamic]
static SELECTOR_NOVEL_COVER: Xpath = parse(SELECT_NOVEL_COVER).unwrap();

// 获取小说最近更新时间
const SELECT_NOVEL_LAST_UPDATED_AT: &str =
    r#"//div[@class="fl"][last()]/table/tbody/tr[2]/td[last()]"#;
#[dynamic]
static SELECTOR_NOVEL_LAST_UPDATED_AT: Xpath = parse(SELECT_NOVEL_LAST_UPDATED_AT).unwrap();

const SELECT_NOVEL_INTRO: &str = r#"//dl[@id="content"]/dd[last()]/p[2]"#;
#[dynamic]
static SELECTOR_NOVEL_INTRO: Xpath = parse(SELECT_NOVEL_INTRO).unwrap();

// 获取html中的属性
macro_rules! elem_attr {
    ($doc: expr, $elem:expr, attr=$name:expr, $or:tt) => {{
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
    ($doc: expr, $elem: expr, $or:tt) => {{
        if let Some(x) = $elem.get_all_text(&$doc) {
            x
        } else {
            $or
        }
    }};
}

pub struct DDSpider {
    db: Arc<DbConn>,
}

impl DDSpider {
    // 返回一个小说元素的迭代器
    pub fn novels_from_page<'a>(page: &'a HtmlDocument) -> impl Iterator + 'a {
        SELECTOR_NOVEL_TABLE
            .apply(&page)
            .ok()
            .and_then(|x| {
                let iter = x
                    .into_iter()
                    // 跳过第一行
                    .skip(1)
                    // 开始获取小说数据
                    .map(|x| SELECTOR_NOVEL_ITEM.apply_to_node(page, x).ok())
                    // 过滤掉空行
                    .filter(|x| x.is_some())
                    // 解除option
                    .map(|x| x.unwrap())
                    // 获取小说信息
                    .map(|x| {
                        // 获取小说名，若没有则失败
                        let name = x.get(0)?.get_all_text(page)?;

                        // 获取小说链接，若没有则失败
                        let link: String = {
                            let a = x.get(0)?.children(page).next()?;

                            elem_attr!(page, a, attr = "href", {
                                return None;
                            })
                        };

                        // 获取小说最新章节名
                        let last_section = x.get(1).and_then(|x| x.get_all_text(page));
                        // 获取作者
                        let author = x
                            .get(2)
                            .and_then(|x| x.get_all_text(page))
                            .unwrap_or(String::from("unknown"));

                        // 获取最近更新时间
                        let last_updated_at: Option<DateTime<Utc>> = x
                            .get(4)
                            .and_then(|x| x.get_all_text(page))
                            .and_then(|x| {
                                DateTime::parse_from_str(
                                    &format!("{x} 21:00:09 +08:00"),
                                    "%Y-%m-%d %H:%M:%S %z",
                                )
                                .ok()
                            })
                            .map(|x| x.into());

                        // 获取完结状态
                        let state = x.get(5).and_then(|x| x.get_all_text(page)).and_then(|x| {
                            let state = match x.trim() {
                                "连载中" => NovelState::Updating,
                                "完本" => NovelState::Finished,
                                _ => NovelState::Updating,
                            };

                            Some(state)
                        });

                        Some((name, link, last_section, author, last_updated_at, state))
                    })
                    // 过滤掉解析失败的
                    .filter(|x| x.is_some());

                Some(iter)
            })
            .into_iter()
            .flatten()
    }

    // 获取封面链接，最近更新时间，简介
    pub fn parse_detail_novel(
        page: &HtmlDocument,
    ) -> (Option<String>, Option<DateTime<Utc>>, Option<String>) {
        fn elem_text(page: &HtmlDocument, selector: &Xpath) -> Option<String> {
            selector
                .apply(page)
                .ok()
                .and_then(|x| x.into_iter().next())
                .and_then(|x| x.get_all_text(page))
        }

        let cover = elem_text(page, &SELECTOR_NOVEL_COVER);

        let updated_at: Option<DateTime<Utc>> = elem_text(page, &SELECTOR_NOVEL_LAST_UPDATED_AT)
            .and_then(|x| {
                DateTime::parse_from_str(&format!("{x} +08:00"), "%Y-%m-%d %H:%M:%S %z").ok()
            })
            .map(|x| x.into());

        let intro = elem_text(page, &SELECTOR_NOVEL_INTRO);

        (cover, updated_at, intro)
    }
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

            let first_url = vec![DATA_URL, &sort.link].concat();
            match pos {
                x @ (Position::Full | Position::First | Position::Last) => {
                    let page = html::parse(&get(&first_url).await?)?;

                    // 处理第一页
                    if matches!(x, Position::First | Position::Full) {
                        let novels = Self::novels_from_page(&page);
                    }

                    let page_num: i32 =
                        if let Some(elem) = SELECTOR_LAST_PAGE.apply(&page)?.into_iter().next() {
                            elem_text!(page, elem, {
                                return Ok::<(), anyhow::Error>(());
                            })
                            .parse()?
                        } else {
                            warn!("没有获取到末尾页数");
                            return Ok::<(), anyhow::Error>(());
                        };
                }
                Position::Specify(_) => {}
                Position::Range(_) => {}
            }

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
