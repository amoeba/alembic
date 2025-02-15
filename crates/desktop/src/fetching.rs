use serde::Deserialize;

use crate::backend::{News, NewsEntry};

// URLs
pub const NEWS_URL : &str = "https://gist.githubusercontent.com/amoeba/cb65198f3c1c59ba945daea6f5484161/raw/054d4d670ea2ee1a85763174ac2cc5ff698a2350/news.json";

pub enum FetchRequest {
    FetchNews,
}

pub enum UpdateMessage {
    NewsUpdate(News),
}
#[derive(Deserialize, Debug)]
pub struct NewsResponseInner {
    pub records: Vec<NewsEntry>,
}

#[derive(Deserialize, Debug)]
pub struct NewsResponse {
    pub news: NewsResponseInner,
}
