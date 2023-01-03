extern crate reqwest;
extern crate scraper;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;

static USERAGENT: &str = "Mozilla/5.0 (Windows NT 10.0; rv:108.0) Gecko/20100101 Firefox/108.0";
// The site returns an empty string unless you have this header
// this is why it worked on `wget` but not `curl`, as well.
static ACCEPT_LANGUAGE: &str = "en-CA,en-US;q=0.7,en;q=0.3";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(reqwest::header::USER_AGENT, USERAGENT.parse().unwrap());
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        ACCEPT_LANGUAGE.parse().unwrap(),
    );

    let reqwest_client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
    let client = ClientBuilder::new(reqwest_client)
        .with(Cache(HttpCache {
            mode: CacheMode::OnlyIfCached,
            manager: CACacheManager::default(),
            options: None,
        }))
        .build();

    let mut res = client
        .get("https://bestkaomoji.com/grinning-face/")
        .send()
        .await?;
    let body = res.text().await?;
    let fragment = Html::parse_document(&body);
    //println!("{:#?}", body);
    let kaomoji_selector = Selector::parse("#kaomojiList").unwrap();
    for li in fragment.select(&kaomoji_selector) {
        let text = li.text().collect::<Vec<_>>();
        println!("{:#?}", text);
    }
    // let mut kaomoji_vec = Vec::new();
    //
    // for kaomoji in fragment.select(&kaomoji_selector) {
    //     kaomoji_vec.push(kaomoji.text().collect::<Vec<_>>()[0].to_string());
    // }
    //
    // let mut file = File::create("kaomoji.txt")?;
    //
    // for kaomoji in kaomoji_vec {
    //     file.write_all(kaomoji.as_bytes())?;
    //     file.write_all(b"\n")?;
    // }

    Ok(())
}
