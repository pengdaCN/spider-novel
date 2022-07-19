use std::fmt::{Display, Formatter};
use std::ops::Range;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tokio::sync::mpsc::Receiver;

use crate::keeper::data::entity::sort::Model as SortModel;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SortID(i64);

impl Into<i64> for SortID {
    fn into(self) -> i64 {
        self.0
    }
}

impl From<i64> for SortID {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl Display for SortID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SortID({})", self.0)
    }
}

#[derive(Debug)]
pub struct Sort {
    pub id: SortID,
    pub name: String,
}

impl From<&'_ SortModel> for Sort {
    fn from(m: &SortModel) -> Self {
        Self {
            id: SortID(m.id),
            name: m.name.clone(),
        }
    }
}

impl From<SortModel> for Sort {
    fn from(m: SortModel) -> Self {
        (&m).into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct NovelID(i64);

impl Into<i64> for NovelID {
    fn into(self) -> i64 {
        self.0
    }
}

impl Into<i64> for &NovelID {
    fn into(self) -> i64 {
        self.0
    }
}

impl From<i64> for NovelID {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl Display for NovelID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NovelID({})", self.0)
    }
}

#[derive(Debug)]
pub enum NovelState {
    Updating,
    Finished,
}

#[derive(Debug)]
pub struct Novel {
    pub id: NovelID,
    pub name: String,
    pub cover: Option<String>,
    pub author: String,
    pub intro: Option<String>,
    pub last_updated_at: Option<DateTime<Utc>>,
    pub last_updated_section_name: Option<String>,
    pub state: Option<NovelState>,
}

// pub enum SectionName {
//     Raw(String),
//     Number(i32),
//     Complex(i32, String),
// }
//
// impl Display for SectionName {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match *self {
//             Self::Raw(ref name) => {
//                 write!(f, "{name}")
//             }
//             Self::Number(n) => {
//                 write!(f, "第{n}章")
//             }
//             Self::Complex(n, ref name) => {
//                 write!(f, "第{n}章: {name}")
//             }
//         }
//     }
// }

#[derive(Debug)]
pub struct Section {
    pub seq: u32,
    pub novel_id: NovelID,
    pub name: String,
    // pub advanced_name: SectionName,
    pub update_at: Option<DateTime<Utc>>,
    pub text: String,
}

#[derive(Debug)]
pub struct Support {
    // 是否支持获取分类
    pub get_sort: bool,
    // 是否支持从分类中获取小说
    pub get_novel_from_sort: bool,
    // 是否支持搜索小说
    pub search_novel: bool,
    // 是否支持精确搜索小说
    pub exact_search_novel: bool,
}

pub enum Position {
    // 获取全部内容
    Full,
    // 获取第一页，或者第一条记录
    First,
    // 获取最后一条，或最有一页记录
    Last,
    // 获取指定条数或者页数的记录
    Specify(i32),
    // 获取指定范围的记录
    Range(Range<i32>),
}

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("network disconnect")]
    Disconnect {
        seq: Option<i32>,
        reason: reqwest::Error,
    },
    #[error("resource not found")]
    ResourceNotFound,
    #[error("parse resource failed")]
    ParseFailed,
    #[error("spider inner error")]
    SpiderInnerFailed(#[from] anyhow::Error),
    #[error("section missing link: {0}")]
    MissSectionLink(i32),
    #[error("section missing content: {0}")]
    MissSectionContent(i32),
}

pub type Result<T> = std::result::Result<T, CrawlError>;

pub trait SpiderMetadata {
    const SUPPORTED: Support;
    // 获取一个网站爬虫的id
    fn id() -> &'static str;
}

#[async_trait]
pub trait Spider: Sync {
    // 获取分类
    fn sorts(&self) -> &Vec<Sort>;

    // 通过分类id获取小说元信息
    async fn novels_by_sort_id(
        &self,
        id: &SortID,
        pos: Position,
    ) -> Result<Receiver<Result<Novel>>>;

    // 通过小说id获取章节和内容
    async fn sections_by_novel_id(
        &self,
        id: &NovelID,
        pos: Position,
    ) -> Result<Receiver<Result<Section>>>;

    // 获取小说元信息
    async fn fetch_novel(&self, id: &NovelID) -> Result<Novel>;

    // 通过小说名字搜索小说
    async fn search(&self, name: &str) -> Result<Vec<Novel>>;

    // 精准搜索
    async fn exact_search(&self, name: &str, author: &str) -> Result<Option<Novel>> {
        Ok(self
            .search(name)
            .await?
            .into_iter()
            .find(|x| x.author == author && x.name.trim() == name))
    }
}
