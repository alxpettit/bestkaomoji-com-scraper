extern crate reqwest;
extern crate scraper;

use lazy_static::lazy_static;
use linya::Progress;
use md5;
use rand::prelude::*;
use rayon::prelude::*;
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::*;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

static USERAGENT: &str = "Mozilla/5.0 (Windows NT 10.0; rv:108.0) Gecko/20100101 Firefox/108.0";
// The site returns an empty string unless you have this header
// this is why it worked on `wget` but not `curl`, as well.
static ACCEPT_LANGUAGE: &str = "en-CA,en-US;q=0.7,en;q=0.3";

lazy_static! {
    static ref RNG: tokio::sync::Mutex<StdRng> = tokio::sync::Mutex::new(StdRng::from_entropy());
}

async fn get_page(
    url: &Url,
    client: &Client,
    delay_range: Option<ops::Range<f32>>,
) -> Result<String, Box<dyn error::Error>> {
    let cache_path = path::PathBuf::from(".").join(".page_cache");
    fs::create_dir_all(&cache_path)?;
    let hash = md5::compute(url.as_str());
    let mut cache_file = cache_path.join(format!("{:?}", hash));
    cache_file.set_extension("html");

    if cache_file.exists() {
        let mut body = String::new();
        fs::File::open(cache_file)?.read_to_string(&mut body)?;
        return Ok(body);
    }

    thread::sleep(Duration::from_secs_f32(
        RNG.lock()
            .await
            .gen_range(delay_range.unwrap_or_else(|| 1.0..10.0)), //delay_range.sl().choose(&*RNG).unwrap(),
    ));
    let res = client.get(url.clone()).send().await?;
    let body = res.text().await?;
    fs::File::create(cache_file)?.write_all(body.as_bytes())?;
    Ok(body)
}

async fn get_links_from_page<'a>(
    selector: &Selector,
    frag: &'a Html,
) -> Result<Vec<&'a str>, Box<dyn error::Error>> {
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
) -> Result<Vec<&'a str>, Box<dyn error::Error>> {
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

async fn on_each_mainpage_link(
    (i, link): (usize, &str),
    client: &Client,
    progress: Arc<Mutex<Progress>>,
    url: &Url,
    selector_category_page: &Selector,
) -> Result<(), Box<dyn error::Error>> {
    let url_category = url.join(link)?;
    let body_category: String = get_page(&url_category, &client, None).await?;
    let frag_category = Html::parse_document(&body_category);
    let kit_links = get_links_from_page(&selector_category_page, &frag_category).await?;

    let kit_bar = progress
        .lock()
        .await
        .bar(kit_links.len(), format!("Downloading {}", link));
    for kit_link in kit_links.iter() {
        let url_kit = url.join(kit_link)?;
        let body_kit = get_page(&url_kit, &client, None).await?;
        let frag_kit = Html::parse_document(&body_kit);

        progress.lock().await.inc_and_draw(&kit_bar, 1);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();
    let progress = Arc::new(Mutex::new(Progress::new()));

    headers.insert(reqwest::header::USER_AGENT, USERAGENT.parse().unwrap());
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        ACCEPT_LANGUAGE.parse().unwrap(),
    );

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
    let url = Url::from_str("https://bestkaomoji.com/").expect("couldn't convert URL");

    let body: String = get_page(&url, &client, None).await?;

    let frag = Html::parse_document(&body);
    let selector_mainpage =
        Selector::parse("#kaomojiSections .kaomojiSection .kaomojiSectionSeeAll a[href]")?;
    let selector_category_page = Selector::parse("ul.kaomojiKitListDefaultView a[href]")?;
    let selector_kaos = Selector::parse("#kaomojiList li")?;
    let mainpage_links = get_links_from_page(&selector_mainpage, &frag).await?;
    mainpage_links.into_par_iter().enumerate().for_each(|data| {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(on_each_mainpage_link(
            data,
            &client,
            progress.clone(),
            &url,
            &selector_category_page,
        ))
        .unwrap();
    });
    //$("#kaomojiList li")

    Ok(())
}
