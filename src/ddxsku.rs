use std::ops::Deref;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{error, warn};
use sea_orm::DbConn;
use skyscraper::html;
use skyscraper::html::HtmlDocument;
use skyscraper::xpath::parse::parse;
use skyscraper::xpath::Xpath;
use static_init::dynamic;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::{channel, Sender};

use crate::common::httputils::get;
use crate::ddxsku::data::novel::Model;
use crate::ddxsku::data::{add_or_recover, add_or_recover_novel, novel_by_id, sort_by_id};
use crate::spider::{
    Novel, NovelID, NovelState, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};

pub mod data;

pub const DATA_URL: &str = "http://www.ddxsku.com";

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

// 获取小说简介
const SELECT_NOVEL_INTRO: &str = r#"//dl[@id="content"]/dd[last()]/p[2]"#;
#[dynamic]
static SELECTOR_NOVEL_INTRO: Xpath = parse(SELECT_NOVEL_INTRO).unwrap();

// 获取小说章节
const SELECT_NOVEL_SECTIONS: &str = r#"//table[@id="at"]/tbody/tr/td/a"#;
#[dynamic]
static SELECTOR_NOVEL_SECTIONS: Xpath = parse(SELECT_NOVEL_SECTIONS).unwrap();

// 获取小说内容
const SELECT_NOVEL_CONTENT: &str = r#"//dd[@id="contents"]"#;
#[dynamic]
static SELECTOR_NOVEL_CONTENT: Xpath = parse(SELECT_NOVEL_CONTENT).unwrap();

// 获取html中的属性
macro_rules! elem_attr {
    ($doc: expr, $elem:expr, attr=$name:expr) => {{
        use skyscraper::html::HtmlNode;
        if let Some(x) = $doc.get_html_node($elem).and_then(|x| match x {
            HtmlNode::Tag(inner) => inner.attributes.get($name),
            _ => None,
        }) {
            Some(x.clone())
        } else {
            None
        }
    }};
    ($doc: expr, $elem:expr, attr=$name:expr, $or:tt) => {{
        if let Some(x) = elem_attr!($doc, $elem, attr = $name) {
            x
        } else {
            $or
        }
    }};
}

// 获取html中文本
macro_rules! elem_text {
    ($doc: expr, $elem: expr, $or:tt) => {{
        if let Some(x) = $elem.get_all_text($doc) {
            x
        } else {
            $or
        }
    }};
}

pub struct SpiderData {
    db: Arc<DbConn>,
}

pub struct DDSpider {
    inner: Arc<SpiderData>,
}

impl Clone for DDSpider {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Deref for DDSpider {
    type Target = SpiderData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DDSpider {
    // 返回一个小说元素的迭代器
    pub fn novels_from_page<'a>(
        page: &'a HtmlDocument,
    ) -> impl Iterator<
        Item = (
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<DateTime<Utc>>,
            Option<NovelState>,
        ),
    > + 'a {
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

                            elem_attr!(page, &a, attr = "href", {
                                return None;
                            })
                        };

                        // 获取小说最新章节名
                        let last_section = x.get(1).and_then(|x| x.get_all_text(page));

                        // 获取小说章节连接
                        let section_link: Option<String> = x
                            .get(1)
                            .and_then(|x| x.children(page).next())
                            .and_then(|x| elem_attr!(page, &x, attr = "href"));

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

                        Some((
                            name,
                            link,
                            last_section,
                            section_link,
                            author,
                            last_updated_at,
                            state,
                        ))
                    })
                    // 过滤掉解析失败的
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap());

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

    async fn parse_novels_from_page<'a>(&self, page: &HtmlDocument) -> Result<Vec<Novel>> {
        let mut novels = Vec::with_capacity(10);
        for (name, link, last_section, section_link, author, mut last_updated_at, state) in
            Self::novels_from_page(&page)
        {
            if section_link.is_none() {
                continue;
            }

            // 获取小说详细信息
            let mut cover = None;
            let mut intro = None;
            if let Some(x) = get(&link)
                .await
                .ok()
                .and_then(|x| html::parse(&x).ok())
                .and_then(|x| Some(Self::parse_detail_novel(&x)))
            {
                last_updated_at = x.1;
                cover = x.0;
                intro = x.2;
            }

            // 分离出小说id
            let novel_id = match link
                .split('/')
                .last()
                .and_then(|x| x.split('.').next())
                .map(|x| String::from(x))
            {
                Some(x) => x,
                None => {
                    warn!("为解析出小说id, link: {}", link);
                    continue;
                }
            };

            // 存储小说信息
            let id = add_or_recover_novel(
                &self.db,
                &name,
                &link,
                &section_link.unwrap(),
                &author,
                &novel_id,
            )
            .await?;

            novels.push(Novel {
                id: id.into(),
                name,
                cover,
                author,
                intro,
                last_updated_at,
                last_updated_section_name: last_section,
                state,
            });
        }

        Ok(novels)
    }

    #[async_recursion]
    async fn send_novels(
        &self,
        link: &str,
        tx: &mut Sender<Result<Novel>>,
        pos: Position,
    ) -> Result<()> {
        match pos {
            x @ (Position::Full | Position::First | Position::Last) => {
                let first_url = vec![DATA_URL, &link].concat();
                let page = html::parse(&get(&first_url).await?)?;

                // 处理第一页
                if matches!(x, Position::First | Position::Full) {
                    for x in self.parse_novels_from_page(&page).await? {
                        if let Err(_) = tx.send(Ok(x)).await {
                            return Ok(());
                        }
                    }
                }

                let page_num: i32 =
                    if let Some(elem) = SELECTOR_LAST_PAGE.apply(&page)?.into_iter().next() {
                        elem_text!(&page, &elem, {
                            return Ok(());
                        })
                        .parse()?
                    } else {
                        warn!("没有获取到末尾页数");
                        return Ok(());
                    };

                match x {
                    Position::First => {
                        return Ok(());
                    }
                    Position::Full => {
                        return self
                            .send_novels(link, tx, Position::Range(2..page_num + 1))
                            .await;
                    }
                    Position::Last => {
                        return self
                            .send_novels(link, tx, Position::Specify(page_num))
                            .await;
                    }
                    _ => unreachable!(),
                }
            }
            Position::Specify(idx) => {
                let page_link = if link.ends_with("full.html") {
                    // 对完本小说特殊处理
                    [
                        DATA_URL,
                        &format!("/modules/article/articlelist.php?fullflag=1&page={}", idx),
                    ]
                    .concat()
                } else {
                    [DATA_URL, &link.replace("1.html", &format!("{}.html", idx))].concat()
                };

                let page = html::parse(&get(&page_link).await?)?;
                for x in self.parse_novels_from_page(&page).await? {
                    if let Err(_) = tx.send(Ok(x)).await {
                        return Ok(());
                    }
                }
            }
            Position::Range(range) => {
                for x in range {
                    let _ = self.send_novels(link, tx, Position::Specify(x)).await?;
                }
            }
        }

        Ok(())
    }

    pub fn sections_from_page<'a>(
        page: &'a HtmlDocument,
    ) -> impl Iterator<Item = (String, Option<String>)> + 'a {
        SELECTOR_NOVEL_SECTIONS
            .apply(page)
            .ok()
            .and_then(|x| {
                let iter = x.into_iter().enumerate().map(|x| {
                    let name =
                        x.1.get_all_text(page)
                            .unwrap_or(format!("unknown-{}", x.0 + 1));
                    let link: Option<String> = elem_attr!(page, &x.1, attr = "href");

                    (name, link)
                });

                Some(iter)
            })
            .into_iter()
            .flatten()
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
            let name: String = elem_text!(&page, &elem, continue);
            let link: String = elem_attr!(page, &elem, attr = "href", continue);

            let id = add_or_recover(&self.db, &name, &link).await?;

            sorts.push(Sort {
                id: id.into(),
                name,
            })
        }

        Ok(sorts)
    }

    async fn novels_by_sort_id(
        &self,
        id: &SortID,
        pos: Position,
    ) -> Result<Receiver<Result<Novel>>> {
        let sort = sort_by_id(&self.db, &id).await?;
        let sort = if let Some(x) = sort {
            x
        } else {
            bail!("Missing sort {}", id)
        };

        let (mut tx, rx) = channel(10);
        let runner = self.clone();
        tokio::spawn(async move {
            let _ = runner.send_novels(&sort.link, &mut tx, pos).await?;
            return Ok::<(), anyhow::Error>(());
        });

        Ok(rx)
    }

    async fn sections_by_novel_id(
        &self,
        id: &NovelID,
        pos: Position,
    ) -> Result<Receiver<Result<Section>>> {
        let novel = match novel_by_id(&self.db, id).await? {
            Some(x) => x,
            None => {
                bail!("Missing novel {}", id)
            }
        };
        let page = html::parse(&get(&novel.section_link).await?)?;

        let (tx, rx) = channel(10);
        let id = id.clone();
        tokio::spawn(async move {
            let mut iter = Self::sections_from_page(&page);
            let sections = match pos {
                Position::Full => iter.collect(),
                pos @ (Position::First | Position::Last | Position::Specify(_)) => {
                    let mut v = Vec::new();

                    let elem = match pos {
                        Position::First => iter.next(),
                        Position::Last => iter.last(),
                        Position::Specify(x) => iter.take(x as usize).next(),
                        _ => unreachable!(),
                    };

                    if let Some(x) = elem {
                        v.push(x);
                    }

                    v
                }
                Position::Range(range) => iter
                    .enumerate()
                    .take_while(|(x, _)| range.contains(&(*x as i32)))
                    .map(|(_, x)| x)
                    .collect(),
            };

            for x in sections {
                if x.1.is_none() {
                    tx.send(Err(anyhow!("Missing content link: {}", x.0)))
                        .await?;
                    continue;
                }

                let link = x.1.unwrap();

                let doc = match get(&link).await {
                    Ok(x) => x,
                    Err(e) => {
                        tx.send(Err(e)).await?;
                        continue;
                    }
                };

                let page = html::parse(&doc)?;
                let content = SELECTOR_NOVEL_CONTENT
                    .apply(&page)
                    .ok()
                    .and_then(|x| x.into_iter().next())
                    .and_then(|x| x.get_all_text(&page));

                match content {
                    Some(doc) => {
                        tx.send(Ok(Section {
                            novel_id: id,
                            name: x.0,
                            update_at: None,
                            text: doc,
                        }))
                        .await?
                    }
                    None => tx.send(Err(anyhow!("Missing content: {}", x.0))).await?,
                }
            }

            return Ok::<(), anyhow::Error>(());
        });

        Ok(rx)
    }

    async fn search(&self, name: &str) -> Result<Option<Vec<Novel>>> {
        todo!()
    }
}
