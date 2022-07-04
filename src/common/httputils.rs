use std::ops::Deref;

use anyhow::Result;
use log::debug;
use reqwest::{Client, header};
use reqwest::header::HeaderValue;
use scraper::Html;
use static_init::dynamic;

#[dynamic]
static CLIENT: Client = {
    let h: header::HeaderMap = vec![
        (header::USER_AGENT,
         HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36 Edg/98.0.1108.43")),
    ].into_iter().collect();

    Client::builder().default_headers(h).build().unwrap()
};

pub async fn get(url: &str) -> Result<Html> {
    let resp = CLIENT.execute(CLIENT.get(url).build().unwrap()).await?;
    let text = resp.text().await?;

    debug!("url = {}; body = {}", url, &text);

    Ok(Html::parse_document(&text))
}


pub struct WrapSend<T>(T);

impl <T> WrapSend<T> {
    pub fn new(x: T) -> Self {
        Self(x)
    }
}

impl<T> Deref for WrapSend<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T> Send for WrapSend<T> {}