use std::{error::Error, io::Read, time::Duration};

use serde::Deserialize;

use crate::backend::{News, NewsEntry};

// URLs
pub const NEWS_URL : &str = "https://gist.githubusercontent.com/amoeba/cb65198f3c1c59ba945daea6f5484161/raw/bf1965a1c6b1bcba923c28e295156e51d2b18ed4/news.json";

// Wrapper around fetch requests
pub enum FetchWrapper<T> {
    NotStarted,
    Started,
    Retrying(u32),
    Success(T),
    Failed(Box<dyn Error + Send + Sync>),
}

// Message types for every background fetch we do
pub enum BackgroundFetchRequest {
    FetchNews,
}

pub enum BackgroundFetchUpdateMessage {
    NewsUpdate(FetchWrapper<News>),
}

// Structs for resources we fetch
#[derive(Deserialize, Debug)]
pub struct NewsResponseInner {
    pub records: Vec<NewsEntry>,
}

#[derive(Deserialize, Debug)]
pub struct NewsResponse {
    pub news: NewsResponseInner,
}

// Actual fetching implementations go here
pub fn fetch_news() -> Result<NewsResponse, Box<dyn Error>> {
    let mut r = reqwest::blocking::get(NEWS_URL)?;

    let mut body: String = String::new();
    r.read_to_string(&mut body)?;

    Ok(serde_json::from_str::<NewsResponse>(&body)?)
}
