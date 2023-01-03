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
    for link in get_links_from_page(&selector_mainpage, &frag).await? {
        let url_cat = url.join(link)?;
        let body_cat: String = get_page(&url_cat, &client).await?;
        let frag_cat = Html::parse_document(&body_cat);
        for kit_link in get_links_from_page(&selector_catpage, &frag_cat).await? {
            println!("{}", kit_link);
        }
    }
    // let fragment = Html::parse_document(&body);
    // //println!("{:#?}", body);
    // let kaomoji_selector = Selector::parse("#kaomojiList").unwrap();
    // for li in fragment.select(&kaomoji_selector) {
    //     let text = li.text().collect::<Vec<_>>();
    //     println!("{:#?}", text);
    // }
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
