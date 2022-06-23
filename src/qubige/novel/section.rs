use anyhow::Result;
use log::info;
use scraper::Selector;
use static_init::dynamic;

use crate::qubige::{document, link};

const SELECT_CONTENT: &str = "div#content";
#[dynamic]
static SELECTOR_CONTENT: Selector = {
    Selector::parse(SELECT_CONTENT).unwrap()
};

#[derive(Debug)]
pub struct Section {
    pub name: String,
    short_link: String,
}

impl Section {
    pub fn new(name: String, short_link: String) -> Self {
        Self {
            name,
            short_link,
        }
    }

    pub async fn contents(&self) -> Result<Option<Vec<String>>> {
        let doc = document(&self.link()).await?;

        if let Some(elem_contents) = doc.select(&SELECTOR_CONTENT).next() {
            let contents = elem_contents.text().map(|s| {
                String::from(s)
            }).collect();

            Ok(Some(contents))
        } else {
            Ok(None)
        }
    }

    fn link(&self) -> String {
        link(&self.short_link)
    }
}