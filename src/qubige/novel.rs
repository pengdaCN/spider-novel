use std::fmt::format;
use std::io::Write;
use std::ops::Range;

use anyhow::Result;
use chrono::prelude::*;
use scraper::Selector;
use static_init::dynamic;

use crate::qubige::novel::section::Section;

pub mod section;

const SELECT_NOVEL: &str = "div.layout.layout2.layout-col2 > ul > li";
const SELECT_LIST: &str = "div.listpage > span.middle > select > option";
const SELECT_NAME: &str = "span.s2 > a";
const SELECT_AUTHOR: &str = "span.s4";
const SELECT_UPDATE_AT: &str = "span.s5";
const SELECT_INTRO: &str = "div.desc.xs-hidden";
const SELECT_SECTION: &str = "div.section-box > ul > li > a";

#[dynamic]
static SELECTOR_NOVEL: Selector = {
    Selector::parse(SELECT_NOVEL).unwrap()
};
#[dynamic]
static SELECTOR_LIST: Selector = {
    Selector::parse(SELECT_LIST).unwrap()
};
#[dynamic]
static SELECTOR_NAME: Selector = {
    Selector::parse(SELECT_NAME).unwrap()
};
#[dynamic]
static SELECTOR_AUTHOR: Selector = {
    Selector::parse(SELECT_AUTHOR).unwrap()
};
#[dynamic]
static SELECTOR_UPDATE_AT: Selector = {
    Selector::parse(SELECT_UPDATE_AT).unwrap()
};
#[dynamic]
static SELECTOR_INTRO: Selector = {
    Selector::parse(SELECT_INTRO).unwrap()
};
#[dynamic]
static SELECTOR_SECTION: Selector = {
    Selector::parse(SELECT_SECTION).unwrap()
};

#[derive(Debug)]
pub struct Novel {
    pub name: String,
    pub author: String,
    pub update_at: Option<DateTime<Utc>>,
    short_link: String,
}

pub enum GetOpt {
    First,
    Full,
    Specify(i32),
    Range(Range<i32>),
}

impl Novel {
    pub fn link(&self) -> String {
        super::link(&self.short_link)
    }

    pub async fn intro(&self) -> Result<Option<String>> {
        // TODO 可以使用缓存，将doc储存一段时间
        let doc = super::document(&self.link()).await?;

        if let Some(elem_intro) = doc.select(&SELECTOR_INTRO).next() {
            if let Some(v) = elem_intro.text().next() {
                Ok(Some(String::from(v)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn sections(&self) -> Result<Option<Vec<Section>>> {
        let doc = super::document(&self.link()).await?;
        let mut sections = Vec::new();
        for elem in doc.select(&SELECTOR_SECTION) {
            let mut name = String::new();
            let mut link = String::new();

            vec![
                || {
                    if let Some(v) = elem.text().next() {
                        name = String::from(v);
                    }
                },
                || {
                    if let Some(v) = elem.value().attr("href") {
                        link = String::from(v);
                    }
                },
            ].into_iter().for_each(|f| {
                f()
            });

            if name.is_empty() || link.is_empty() {
                continue;
            }

            sections.push(section::Section::new(name, link));
        }
        Ok(Some(sections))
    }
}

fn links(sort: &str, opt: GetOpt) -> Option<Vec<String>> {
    match opt {
        GetOpt::First => {
            let links = vec![String::from(sort)];
            Some(links)
        }
        GetOpt::Full => {
            None
        }
        GetOpt::Specify(idx) => {
            if idx < 1 {
                return None;
            }

            if idx == 1 {
                return links(sort, GetOpt::First);
            }

            Some(vec![format!("{}/index_{}.html", sort, idx)])
        }
        GetOpt::Range(range) => {
            let mut _links = Vec::new();

            for idx in range {
                if let Some(link) = links(sort, GetOpt::Specify(idx)) {
                    _links.push(link.into_iter().next().unwrap());
                }
            }

            Some(_links)
        }
    }
}

pub async fn from_sort(sort: &str, opt: GetOpt) -> Result<Vec<Novel>> {
    let links = links(sort, opt);

    let links = if let Some(have_links) = links {
        have_links
    } else {
        // 爬取网页，获取去全部数据
        let doc = super::document(sort).await?;

        let mut links = Vec::new();
        for elem in doc.select(&SELECTOR_LIST) {
            if let Some(link) = elem.value().attr("value") {
                links.push(String::from(link))
            }
        }

        links
    };

    let mut novels = Vec::new();
    for link in links {
        let page_link = super::link(&link);
        let page_doc = super::document(&page_link).await?;

        for novel_elem in page_doc.select(&SELECTOR_NOVEL) {
            // 获取书名，链接
            let (name, link) = {
                if let Some(name_elem) = novel_elem.select(&SELECTOR_NAME).next() {
                    let name = {
                        if let Some(name) = name_elem.text().next() {
                            String::from(name)
                        } else {
                            continue;
                        }
                    };

                    let link = {
                        if let Some(link) = name_elem.value().attr("href") {
                            String::from(link)
                        } else {
                            continue;
                        }
                    };

                    (name, link)
                } else {
                    continue;
                }
            };

            let mut author = String::new();
            let mut update_at = String::new();
            // 获取作者名，更新事件
            for (value, select) in {
                let mut arr = Vec::<(&mut String, &Selector)>::new();
                arr.push((&mut author, &SELECTOR_AUTHOR));
                arr.push((&mut update_at, &SELECTOR_AUTHOR));

                arr
            } {
                if let Some(elem) = novel_elem.select(select).next() {
                    if let Some(v) = elem.text().next() {
                        *value = String::from(v);
                    }
                } else {
                    continue;
                }
            }

            let update_at: Option<DateTime<Utc>> = {
                if let Ok(date_time) = DateTime::parse_from_str(&format!("{} +08:00", update_at), "%Y-%m-%d %z") {
                    Some(date_time.into())
                } else {
                    None
                }
            };

            novels.push(Novel {
                name,
                author,
                update_at,
                short_link: link,
            })
        }
    }

    Ok(novels)
}