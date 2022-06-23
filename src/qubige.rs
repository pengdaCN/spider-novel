use anyhow::Result;
use log::{debug, info};
use reqwest::{Client, header};
use reqwest::header::HeaderValue;
use scraper::Html;
use static_init::dynamic;

pub mod sort;
pub mod novel;

const LINK_BASE: &str = "https://www.qubige.com/";

#[dynamic]
static CLIENT: Client = {
    let h: header::HeaderMap = vec![
        (header::USER_AGENT,
         HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36 Edg/98.0.1108.43")),
    ].into_iter().collect();

    Client::builder()
        .default_headers(h)
        .build()
        .unwrap()
};

pub(crate) fn link(path: &str) -> String {
    let mut link = String::from(LINK_BASE);
    link.push_str(path);

    link
}

pub(crate) async fn document(url: &str) -> Result<Html> {
    let resp = CLIENT.execute(CLIENT.get(url).build().unwrap()).await?;
    let text = resp.text().await?;

    debug!("url = {}; body = {}", url, &text);

    Ok(Html::parse_document(&text))
}