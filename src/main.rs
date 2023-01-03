extern crate reqwest;
extern crate scraper;

use md5;
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

static USERAGENT: &str = "Mozilla/5.0 (Windows NT 10.0; rv:108.0) Gecko/20100101 Firefox/108.0";
// The site returns an empty string unless you have this header
// this is why it worked on `wget` but not `curl`, as well.
static ACCEPT_LANGUAGE: &str = "en-CA,en-US;q=0.7,en;q=0.3";

async fn get_page(url: &Url, client: &Client) -> Result<String, Box<dyn Error>> {
    let cache_path = PathBuf::from(".").join(".page_cache");
    fs::create_dir_all(&cache_path)?;
    let hash = md5::compute(url.as_str());
    let mut cache_file = cache_path.join(format!("{:?}", hash));
    cache_file.set_extension("html");

    if cache_file.exists() {
        let mut body = String::new();
        File::open(cache_file)?.read_to_string(&mut body)?;
        return Ok(body);
    }

    let res = client.get(url.clone()).send().await?;
    let body = res.text().await?;
    File::create(cache_file)?.write_all(body.as_bytes())?;
    Ok(body)
}

async fn get_links_from_page<'a>(
    selector: &Selector,
    frag: &'a Html,
) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let mut ret: Vec<&'a str> = Vec::new();
    for link_element in frag.select(&selector) {
        let link = link_element
            .value()
            .attr("href")
            .ok_or("Could not extract link value from href!")?;
        ret.push(link.clone());
    }
    Ok(ret)
}

async fn get_kaos_from_page<'a>(
    selector: &Selector,
    frag: &'a Html,
) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let mut ret: Vec<&'a str> = Vec::new();
    for link_element in frag.select(&selector) {
        let link = link_element
            .value()
            .attr("href")
            .ok_or("Could not extract link value from href!")?;
        ret.push(link.clone());
    }
    Ok(ret)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(reqwest::header::USER_AGENT, USERAGENT.parse().unwrap());
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        ACCEPT_LANGUAGE.parse().unwrap(),
    );

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
    let url = Url::from_str("https://bestkaomoji.com/").expect("couldn't convert URL");

    let body: String = get_page(&url, &client).await?;

    let frag = Html::parse_document(&body);
    let selector_mainpage =
        Selector::parse("#kaomojiSections .kaomojiSection .kaomojiSectionSeeAll a[href]")?;
    let selector_catpage = Selector::parse("ul.kaomojiKitListDefaultView a[href]")?;
    let selector_kaos = Selector::parse("#kaomojiList li")?;
    for link in get_links_from_page(&selector_mainpage, &frag).await? {
        let url_category = url.join(link)?;
        let body_category: String = get_page(&url_category, &client).await?;
        let frag_category = Html::parse_document(&body_category);
        for kit_link in get_links_from_page(&selector_catpage, &frag_category).await? {
            let url_kit = url.join(kit_link)?;
            let body_kit = get_page(&url_kit, &client).await?;
            let frag_kit = Html::parse_document(&body_kit);
            //for kao in get_kao_from_page(&selector_kaos, &frag_kit) {}
            std::thread::sleep(Duration::from_secs_f32(2.))
        }
    }
    //$("#kaomojiList li")

    Ok(())
}
