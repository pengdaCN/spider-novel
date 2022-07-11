use std::ops::Deref;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{error, warn};
use sea_orm::DbConn;
use static_init::dynamic;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::{channel, Sender};

use crate::common::doc;
use crate::common::doc::{WrapDocument, WrapSelection};
use crate::common::httputils::get;
use crate::ddxsku::data::novel::Model;
use crate::ddxsku::data::{add_or_recover, add_or_recover_novel, novel_by_id, sort_by_id};
use crate::spider::{
    Novel, NovelID, NovelState, Position, Section, Sort, SortID, Spider, SpiderMetadata, Support,
};

pub mod data;

pub const DATA_URL: &str = "http://www.ddxsku.com";

// 获取小说分类
const SELECT_SORT: &str = r#"div.main.m_menu > ul > li"#;

// 获取最后一条分页
const SELECT_LAST_PAGE: &str = r#"//a[@class="last"]"#;

// 获取小说列表
const SELECT_NOVEL_TABLE: &str = r#"tbody > tr"#;

// 获取列表中的小说条目
const SELECT_NOVEL_ITEM: &str = r#"td"#;

// 获取小说封面链接
const SELECT_NOVEL_COVER: &str = r#"//div.fl:first-of-type img"#;

// 获取小说最近更新时间
const SELECT_NOVEL_LAST_UPDATED_AT: &str =
    r#"div.fl:last-of-type > table > tbody > tr:nth-of-type(2) > td:last-of-type"#;

// 获取小说简介
const SELECT_NOVEL_INTRO: &str = r#"dl#content > dd:last-of-type > p:nth-of-type(2)"#;

// 获取小说章节
const SELECT_NOVEL_SECTIONS: &str = r#"table#at > tbody > tr > td > a"#;

// 获取小说内容
const SELECT_NOVEL_CONTENT: &str = r#"dd#contents"#;

// 获取html中的属性
macro_rules! elem_attr {
    ($doc: expr, attr=$name:expr, $or:tt) => {{
        if let Some(x) = $doc.attr($name) {
            x
        } else {
            $or
        }
    }};
}

// 获取html中文本
macro_rules! elem_text {
    ($doc: expr, $or:tt) => {{
        if let Some(x) = $doc.text() {
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
    pub fn new(db: Arc<DbConn>) -> Self {
        Self {
            inner: Arc::new(SpiderData { db }),
        }
    }

    // 返回一个小说元素的迭代器
    fn novels_from_page<'a>(
        page: &'a WrapDocument,
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
        page.select(SELECT_NOVEL_TABLE)
            .iter()
            .skip(1)
            .map(|x| {
                let r: Vec<WrapSelection<'_>> = x.select(SELECT_NOVEL_ITEM).iter().collect();
                r
            })
            .map(|x| {
                // 获取小说名，若没有则失败
                let name = elem_text!(x.get(0)?, {
                    return None;
                });
                // 获取小说链接，若没有则失败
                let link = elem_attr!(x.get(0)?.children(), attr = "href", {
                    return None;
                });

                // 获取小说最新章节名
                let last_section = x.get(1).and_then(|x| x.text());

                // 获取小说章节连接
                let section_link: Option<String> = x.get(1).and_then(|x| x.children().attr("href"));

                // 获取作者
                let author = x
                    .get(2)
                    .and_then(|x| x.text())
                    .unwrap_or(String::from("unknown"));

                // 获取最近更新时间
                let last_updated_at: Option<DateTime<Utc>> = x
                    .get(4)
                    .and_then(|x| x.text())
                    .and_then(|x| {
                        DateTime::parse_from_str(
                            &format!("{x} 21:00:09 +08:00"),
                            "%Y-%m-%d %H:%M:%S %z",
                        )
                        .ok()
                    })
                    .map(|x| x.into());

                // 获取完结状态
                let state = x.get(5).and_then(|x| x.text()).and_then(|x| {
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
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
    }

    // 获取封面链接，最近更新时间，简介
    fn parse_detail_novel(
        page: &WrapDocument,
    ) -> (Option<String>, Option<DateTime<Utc>>, Option<String>) {
        let cover = page.select(SELECT_NOVEL_ITEM).attr("href");

        let updated_at: Option<DateTime<Utc>> = page
            .select(SELECT_NOVEL_LAST_UPDATED_AT)
            .text()
            .and_then(|x| {
                DateTime::parse_from_str(&format!("{x} +08:00"), "%Y-%m-%d %H:%M:%S %z").ok()
            })
            .map(|x| x.into());

        let intro = page.select(SELECT_NOVEL_INTRO).text();

        (cover, updated_at, intro)
    }

    async fn parse_novels_from_page<'a>(&self, page: &WrapDocument) -> Result<Vec<Novel>> {
        let mut novels = Vec::with_capacity(10);
        for (name, link, last_section, section_link, author, mut last_updated_at, state) in
            Self::novels_from_page(page)
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
                .and_then(|x| Some(Self::parse_detail_novel(&WrapDocument::parse(&x))))
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

    async fn handle_page(&self, page: &WrapDocument, tx: &mut Sender<Result<Novel>>) -> Result<()> {
        for x in self.parse_novels_from_page(page).await? {
            if let Err(_) = tx.send(Ok(x)).await {
                return Ok(());
            }
        }

        Ok(())
    }

    #[async_recursion]
    async fn send_novels(
        &self,
        link: &str,
        tx: &mut Sender<Result<Novel>>,
        mut pos: Position,
    ) -> Result<()> {
        match pos {
            x @ (Position::Full | Position::First | Position::Last) => {
                let first_url = vec![DATA_URL, &link].concat();
                let page = WrapDocument::parse(&get(&first_url).await?);

                // 处理第一页
                if matches!(x, Position::First) {
                    self.handle_page(&page, tx).await?
                }

                let page_num: i32 = if let Some(last) = page.select(SELECT_LAST_PAGE).text() {
                    last.parse()?
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

                let page = WrapDocument::parse(&get(&page_link).await?);
                self.handle_page(&page, tx).await?
            }
            Position::Range(range) => {
                for x in range {
                    let _ = self.send_novels(link, tx, Position::Specify(x)).await?;
                }
            }
        }

        Ok(())
    }

    fn sections_from_page<'a>(
        page: &'a WrapDocument,
    ) -> impl Iterator<Item = (String, Option<String>)> + 'a {
        page.select(SELECT_NOVEL_SECTIONS)
            .iter()
            .enumerate()
            .map(|x| {
                let name = x.1.text().unwrap_or(format!("unknown-{}", x.0 + 1));
                let link = x.1.attr("href");

                (name, link)
            })
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
        let page = WrapDocument::parse(&raw_page);

        for elem in page.select(SELECT_SORT).iter() {
            let name = elem_text!(elem, continue);
            let link = elem_attr!(elem.children(), attr = "href", continue);

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
        let page = WrapDocument::parse(&get(&novel.section_link).await?);

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

                let page = WrapDocument::parse(&doc);
                let content = page.select(SELECT_NOVEL_CONTENT).text();

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
