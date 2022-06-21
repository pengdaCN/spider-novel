use anyhow::Result;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<()> {
    let resp = reqwest::get("https://www.qubige.com/sort/").await?;
    let text = resp.text().await?;

    let document = Html::parse_document(&text);
    let select_sort = Selector::parse("div.cmd-bd > a").unwrap();

    for elem in document.select(&select_sort) {
        let name = {
            if let Some(name) = elem.text().next() {
                String::from(name)
            } else {
                continue;
            }
        };
        println!("name = {}, href = {}", name, elem.value().attr("href").unwrap());
    }

    Ok(())
}