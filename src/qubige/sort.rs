use scraper::{Html, Selector};
use anyhow::Result;
use static_init::dynamic;

const LINK_SORT: &str = "https://www.qubige.com/sort/";
const SELECT_SORT: &str = "div.cmd-bd > a";

#[dynamic]
static SELECTOR_SORT: Selector = {
    Selector::parse(SELECT_SORT).unwrap()
};

#[derive(Debug)]
pub struct Sort {
    pub name: String,
    pub link: String,
}

impl Sort {
    pub fn link(&self) -> String {
        super::link(&self.link)
    }
}


pub async fn get_sort() -> Result<Vec<Sort>> {
    let mut sorts = Vec::new();

    let doc = super::document(LINK_SORT).await?;
    let select_sort = Selector::parse(SELECT_SORT).unwrap();

    for elem in doc.select(&select_sort) {
        let name = {
            if let Some(name) = elem.text().next() {
                String::from(name)
            } else {
                continue;
            }
        };

        let href = {
            if let Some(link) = elem.value().attr("href") {
                String::from(link)
            } else {
                continue;
            }
        };

        sorts.push(Sort {
            name,
            link: href,
        })
    }

    Ok(sorts)
}