use reqwest::header::HeaderValue;
use reqwest::{header, Client};
use static_init::dynamic;

#[dynamic]
pub static CLIENT: Client = {
    let h: header::HeaderMap = vec![
        (header::USER_AGENT,
         HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36 Edg/98.0.1108.43")),
    ].into_iter().collect();

    Client::builder().default_headers(h).build().unwrap()
};

pub async fn get(url: &str) -> reqwest::Result<String> {
    let resp = CLIENT.execute(CLIENT.get(url).build().unwrap()).await?;
    let text = resp.text().await?;

    Ok(text)
}
