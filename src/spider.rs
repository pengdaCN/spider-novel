use std::fmt::{Display, Formatter};
use std::ops::Range;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::Receiver;

use crate::keeper::data::entity::sort::Model as SortModel;

#[derive(Debug, Copy, Clone)]
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

pub enum NovelState {
    Updating,
    Finished,
}

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

pub struct SectionID(i32);

impl Into<i32> for SectionID {
    fn into(self) -> i32 {
        self.0
    }
}

impl From<i32> for SectionID {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl Display for SectionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SectionID({})", self.0)
    }
}

#[derive(Debug)]
pub struct Section {
    // pub id: SectionID,
    pub novel_id: NovelID,
    pub name: String,
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
}

pub enum Position {
    Full,
    First,
    Last,
    Specify(i32),
    Range(Range<i32>),
}

pub trait SpiderMetadata {
    const SUPPORTED: Support;
    // 获取一个网站爬虫的id
    fn id() -> &'static str;
}

#[async_trait]
pub trait Spider: Sync {
    // 获取分类
    async fn sorts(&self) -> Result<Vec<Sort>> {
        unimplemented!()
    }

    // 通过分类id获取小说元信息
    // TODO 修改返回值为result类型
    #[allow(unused_variables)]
    async fn novels_by_sort_id(
        &self,
        id: &SortID,
        pos: Position,
    ) -> Result<Receiver<Result<Novel>>> {
        unimplemented!()
    }

    // 通过小说id获取章节和内容
    #[allow(unused_variables)]
    async fn sections_by_novel_id(
        &self,
        id: &NovelID,
        pos: Position,
    ) -> Result<Receiver<Result<Section>>> {
        unimplemented!()
    }

    // 通过小说名字搜索小说
    #[allow(unused_variables)]
    async fn search(&self, name: &str) -> Result<Option<Vec<Novel>>> {
        unimplemented!()
    }
}
