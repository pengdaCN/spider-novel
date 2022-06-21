use anyhow::Result;
use scraper::{Html, Selector};

const LINK_SORT: &str = "https://www.qubige.com/sort/";
const SELECT_SORT: &str = "div.cmd-bd > a";

pub struct Sort {
    pub name: String,
    pub link: String,
}

async fn get_sort() -> Result<Sort> {
    let sorts = Vec::new();

    let resp = reqwest::get(LINK_SORT).await?;
    let text = resp.text().await?;

    let doc = Html::parse_document(&text);
    let select_sort = Selector::parse(&SELECT_SORT).unwrap();

    Ok(sorts)
}