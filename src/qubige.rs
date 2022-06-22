use scraper::Html;
use anyhow::Result;

pub mod sort;
pub mod novel;

const LINK_BASE: &str = "https://www.qubige.com/";

pub(crate) fn link(path: &str) -> String {
    let mut link = String::from(LINK_BASE);
    link.push_str(path);

    link
}

pub(crate) async fn document(url: &str) -> Result<Html> {
    let resp = reqwest::get(url).await?;
    let text = resp.text().await?;

    Ok(Html::parse_document(&text))
}