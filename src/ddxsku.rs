use std::sync::Arc;

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{info, warn};
use sea_orm::{DbConn, TransactionTrait};
use serde_json::json;
use tera::{Context, Tera};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Semaphore;

use crate::common::doc::{WrapDocument, WrapSelection};
use crate::common::httputils::get;
use crate::common::sender::WrapSender;
use crate::ddxsku::data::{add_or_recover, add_or_recover_novel, clear_sort, novel_by_id, sorts};
use crate::spider;
use crate::spider::{
    CrawlError, Novel, NovelID, NovelState, Position, Section, Sort, SortID, Spider,
    SpiderMetadata, Support,
};

pub mod data;

// 默认并发大小
const DEFAULT_CONCURRENT_MAX: usize = 100;

// 网站地址
const DATA_URL: &str = "http://www.ddxsku.com";

// 获取最后一条分页
const SELECT_LAST_PAGE: &str = r#"a.last"#;

// 获取小说列表
const SELECT_NOVEL_TABLE: &str = r#"tbody > tr"#;

// 获取列表中的小说条目
const SELECT_NOVEL_ITEM: &str = r#"td"#;

// 获取小说封面链接
const SELECT_NOVEL_COVER: &str = r#"dl div.fl:first-of-type img"#;

// 获取小说最近更新时间
const SELECT_NOVEL_LAST_UPDATED_AT: &str =
    r#"div.fl:last-of-type > table > tbody > tr:nth-of-type(2) > td:last-of-type"#;

// 获取小说简介
const SELECT_NOVEL_INTRO: &str = r#"dl#content > dd:last-of-type > p:nth-of-type(2)"#;

// 获取最新章节
const SELECT_NOVEL_LAST_SECTION: &str = r"dl#content > dd:last-of-type > p > a";

// 获取小说状态
const SELECT_NOVEL_STATE: &str =
    "dl#content > dd:nth-of-type(2) > div > table > tbody > tr:first-of-type > td:last-of-type";

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

#[derive(Clone)]
pub struct DDSpider {
    db: Arc<DbConn>,
    smp: Arc<Semaphore>,
    templates: Arc<(Tera, Vec<Sort>)>,
}

pub struct SortEntity {
    pub name: String,
    pub link: String,
}

impl DDSpider {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self {
            db,
            smp: Arc::new(Semaphore::new(DEFAULT_CONCURRENT_MAX)),
            templates: Arc::new((Tera::default(), vec![])),
        }
    }

    pub async fn set_sort(&mut self, data: &Vec<SortEntity>) -> Result<()> {
        // 开启事务
        let txn = self.db.begin().await?;

        // 删除原来的数据
        clear_sort(&txn).await?;

        // 新添加的数据的模板引擎
        let mut engine = Tera::default();
        let mut sorts = Vec::new();
        // 插入新的数据
        for x in data {
            // 添加模板
            engine.add_raw_template(&x.name, &x.link)?;
            // 写入到数据
            let id = add_or_recover(&txn, &x.name, &x.link).await?;
            sorts.push(Sort {
                id: id.into(),
                name: String::from(&x.name),
            });
        }

        // 替换完成
        txn.commit().await?;
        self.templates = Arc::new((engine, sorts));

        Ok(())
    }

    pub async fn load_sorts(&mut self) -> Result<()> {
        let db: &DbConn = &self.db;
        let x = sorts(db).await?;

        let mut engine = Tera::default();
        let mut sorts = Vec::new();
        for x in x {
            engine.add_raw_template(&x.name, &x.link)?;
            sorts.push(Sort {
                id: x.id.into(),
                name: String::from(&x.name),
            });
        }

        self.templates = Arc::new((engine, sorts));

        Ok(())
    }

    fn render_sort_link(&self, sort_id: &SortID, idx: i32) -> spider::Result<String> {
        let name = self
            .templates
            .1
            .iter()
            .find(|x| x.id == *sort_id)
            .take()
            .map(|x| &x.name)
            .ok_or(CrawlError::ResourceNotFound)?;

        let link = self
            .templates
            .0
            .render(
                name,
                &Context::from_serialize(&json!({
                    "page": idx,
                }
                ))
                .unwrap(),
            )
            .map_err(|e| CrawlError::SpiderInnerFailed(e.into()))?;

        Ok(link)
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

    // 解析页面信息 封面链接，最近更新时间，最近更新章节，简介
    fn parse_detail_novel2(
        page: &WrapDocument,
    ) -> (
        Option<String>,
        Option<DateTime<Utc>>,
        Option<String>,
        Option<NovelState>,
        Option<String>,
    ) {
        let cover = page.select(SELECT_NOVEL_COVER).attr("href");

        let updated_at: Option<DateTime<Utc>> = page
            .select(SELECT_NOVEL_LAST_UPDATED_AT)
            .text()
            .and_then(|x| {
                DateTime::parse_from_str(&format!("{x} +08:00"), "%Y-%m-%d %H:%M:%S %z").ok()
            })
            .map(|x| x.into());

        let intro = page.select(SELECT_NOVEL_INTRO).text();

        let last_section = page.select(SELECT_NOVEL_LAST_SECTION).text();

        let state = page.select(SELECT_NOVEL_STATE).text().and_then(|x| {
            let state = match x.trim() {
                "连载中" => NovelState::Updating,
                "完本" => NovelState::Finished,
                _ => NovelState::Updating,
            };

            Some(state)
        });

        (cover, updated_at, last_section, state, intro)
    }

    // 获取封面链接，最近更新时间，简介
    fn parse_detail_novel(
        page: &WrapDocument,
    ) -> (Option<String>, Option<DateTime<Utc>>, Option<String>) {
        let (cover, updated_at, _, _, intro) = Self::parse_detail_novel2(page);

        (cover, updated_at, intro)
    }

    async fn parse_novels_from_page(&self, page: &WrapDocument) -> Vec<Result<Novel>> {
        let mut handlers = Vec::with_capacity(10);

        for (name, link, last_section, section_link, author, mut last_updated_at, state) in
            Self::novels_from_page(page)
        {
            let db = self.db.clone();
            let permit = self.smp.clone().acquire_owned().await.unwrap();

            let handler = tokio::spawn(async move {
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

                // 存储小说信息
                let id =
                    add_or_recover_novel(&db, &name, &link, &section_link.unwrap(), &author, "0")
                        .await?;

                drop(permit);

                Ok::<Novel, anyhow::Error>(Novel {
                    id: id.into(),
                    name,
                    cover,
                    author,
                    intro,
                    last_updated_at,
                    last_updated_section_name: last_section,
                    state,
                })
            });

            handlers.push(handler);
        }

        let mut novels = Vec::with_capacity(10);

        for join_handler in handlers {
            let novel = join_handler
                .await
                .expect("parse_novels_from_page task panic");

            novels.push(novel);
        }

        novels
    }

    #[async_recursion]
    async fn send_novels(&self, id: &SortID, tx: Sender<spider::Result<Novel>>, pos: Position) {
        macro_rules! send_err_abort {
            ($expression:expr, $tx: expr) => {
                match $expression {
                    Ok(x) => x,
                    Err(e) => {
                        let _ = $tx.send(Err(e)).await;

                        return;
                    }
                }
            };
        }

        macro_rules! send_iter_or_abort {
            ($elems: expr, $tx: expr) => {
                for x in $elems {
                    if $tx.send(x).await.is_err() {
                        return;
                    }
                }
            };
        }

        match pos {
            x @ (Position::Full | Position::First | Position::Last) => {
                let first_url = send_err_abort!(self.render_sort_link(id, 1), tx);

                let page = WrapDocument::parse(&send_err_abort!(
                    get(&first_url).await.map_err(|e| CrawlError::Disconnect {
                        seq: Some(1),
                        reason: e,
                    }),
                    tx
                ));

                // 处理第一页
                if matches!(x, Position::First) {
                    if tx.is_closed() {
                        return;
                    }

                    send_iter_or_abort!(
                        self.parse_novels_from_page(&page)
                            .await
                            .into_iter()
                            .map(|x| { x.map_err(|e| CrawlError::SpiderInnerFailed(e.into())) })
                            .collect::<Vec<_>>(),
                        tx
                    );
                }

                let page_num: i32 = if let Some(last) = page.select(SELECT_LAST_PAGE).text() {
                    match last.parse() {
                        Ok(x) => x,
                        Err(_) => {
                            let _ = tx.send(Err(CrawlError::ParseFailed)).await;
                            return;
                        }
                    }
                } else {
                    warn!("没有获取到末尾页数; url: {first_url}");
                    return;
                };

                match x {
                    Position::First => {
                        return;
                    }
                    Position::Full => {
                        return self
                            .send_novels(id, tx, Position::Range(2..page_num + 1))
                            .await;
                    }
                    Position::Last => {
                        return self.send_novels(id, tx, Position::Specify(page_num)).await;
                    }
                    _ => unreachable!(),
                }
            }

            Position::Specify(idx) => {
                let page_link = send_err_abort!(self.render_sort_link(id, idx), tx);

                if tx.is_closed() {
                    return;
                }

                let page = WrapDocument::parse(&send_err_abort!(
                    get(&page_link).await.map_err(|e| CrawlError::Disconnect {
                        seq: Some(idx),
                        reason: e,
                    }),
                    tx
                ));

                send_iter_or_abort!(
                    self.parse_novels_from_page(&page)
                        .await
                        .into_iter()
                        .map(|x| { x.map_err(|e| CrawlError::SpiderInnerFailed(e.into())) })
                        .collect::<Vec<_>>(),
                    tx
                );
            }
            Position::Range(range) => {
                let smp = Arc::new(Semaphore::new(DEFAULT_CONCURRENT_MAX));
                for x in range {
                    let id = id.clone();
                    let tx = tx.clone();
                    let runner = self.clone();

                    let permit = smp.clone().acquire_owned().await.unwrap();
                    // 对并发执行做限制
                    info!("开始爬取页面编号 {x}");
                    tokio::spawn(async move {
                        runner.send_novels(&id, tx, Position::Specify(x)).await;

                        drop(permit);
                    });
                }
            }
        }
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
        exact_search_novel: true,
    };

    fn id() -> &'static str {
        DATA_URL
    }
}

#[async_trait]
impl Spider for DDSpider {
    fn sorts(&self) -> &Vec<Sort> {
        &self.templates.1
    }

    async fn novels_by_sort_id(
        &self,
        id: &SortID,
        pos: Position,
    ) -> spider::Result<Receiver<spider::Result<Novel>>> {
        // 检查sort id 是否存在
        let _ = self.render_sort_link(id, 1)?;

        let (tx, rx) = channel(10);
        let runner = self.clone();
        let id = id.clone();
        tokio::spawn(async move {
            runner.send_novels(&id, tx, pos).await;
        });

        Ok(rx)
    }

    async fn sections_by_novel_id(
        &self,
        id: &NovelID,
        pos: Position,
    ) -> spider::Result<Receiver<spider::Result<Section>>> {
        macro_rules! send_or_abort {
            ($tx:expr, $val: expr) => {
                if let Err(_) = $tx.send($val).await {
                    return;
                }
            };
        }

        let novel = match novel_by_id(&self.db, id).await? {
            Some(x) => x,
            None => {
                return Err(CrawlError::ResourceNotFound);
            }
        };
        let page = WrapDocument::parse(&get(&novel.section_link).await.map_err(|e| {
            CrawlError::Disconnect {
                seq: None,
                reason: e,
            }
        })?);

        let (tx, rx) = channel(50);
        let id = id.clone();
        let smp = self.smp.clone();
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
                    .filter(|(idx, _)| range.contains(&((*idx as i32) + 1)))
                    .map(|(_, x)| x)
                    .collect(),
            };

            let order_tx = WrapSender::wrap(tx.clone());
            for x in sections.into_iter().enumerate() {
                let seq = x.0;
                let info = x.1;
                if info.1.is_none() {
                    send_or_abort!(tx, Err(CrawlError::MissSectionLink(seq as i32)));
                    continue;
                }

                let link = info.1.unwrap();

                let permit = smp.clone().acquire_owned().await.unwrap();
                let tx = match order_tx.permit_owned().await {
                    Ok(x) => x,
                    Err(_) => return,
                };

                // 并发执行
                tokio::spawn(async move {
                    let doc = match get(&link).await {
                        Ok(x) => x,
                        Err(e) => {
                            tx.send(Err(CrawlError::Disconnect {
                                seq: Some(seq as i32),
                                reason: e,
                            }))
                            .await;
                            return;
                        }
                    };

                    let page = WrapDocument::parse(&doc);
                    let content = page.select(SELECT_NOVEL_CONTENT).text();

                    match content {
                        Some(doc) => {
                            tx.send(Ok(Section {
                                seq: seq as u32,
                                novel_id: id,
                                name: info.0,
                                update_at: None,
                                text: doc,
                            }))
                            .await;
                        }
                        None => {
                            tx.send(Err(CrawlError::MissSectionContent(seq as i32)))
                                .await;
                        }
                    }

                    drop(permit);
                });
            }
        });

        Ok(rx)
    }

    // 获取小说元数据
    async fn fetch_novel(&self, id: &NovelID) -> spider::Result<Novel> {
        let novel = match novel_by_id(&self.db, id).await? {
            Some(x) => x,
            None => {
                return Err(CrawlError::ResourceNotFound);
            }
        };

        let doc = get(&novel.raw_link)
            .await
            .map_err(|e| CrawlError::Disconnect {
                seq: None,
                reason: e,
            })?;

        let page = WrapDocument::parse(&doc);

        let (cover, updated_at, last_section, state, intro) = Self::parse_detail_novel2(&page);

        Ok(Novel {
            id: novel.id.into(),
            name: novel.name,
            cover,
            author: novel.author,
            intro,
            last_updated_at: updated_at,
            last_updated_section_name: last_section,
            state,
        })
    }

    async fn search(&self, name: &str) -> spider::Result<Vec<Novel>> {
        todo!()
    }

    async fn exact_search(&self, name: &str, author: &str) -> spider::Result<Novel> {
        todo!()
    }
}
