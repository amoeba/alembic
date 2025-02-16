use std::{error::Error, io::Read, time::Duration};

use serde::Deserialize;

use crate::backend::{CommunityServers, News, NewsEntry};

// URLs
pub const NEWS_URL: &str = "https://gist.githubusercontent.com/amoeba/cb65198f3c1c59ba945daea6f5484161/raw/ed4cd3733bbc3ec92164162fbe247ebb190fc377/news.json";
pub const COMMUNITY_SERVERS_LIST_URL: &str =
    "https://raw.githubusercontent.com/acresources/serverslist/refs/heads/master/Servers.xml";

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
    FetchCommunityServers,
}

pub enum BackgroundFetchUpdateMessage {
    NewsUpdate(FetchWrapper<News>),
    CommunityServersUpdate(FetchWrapper<CommunityServers>),
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

    let mut buf: String = String::new();
    r.read_to_string(&mut buf)?;

    Ok(serde_json::from_str::<NewsResponse>(&buf)?)
}

pub fn fetch_community_servers_list() -> Result<CommunityServers, Box<dyn Error>> {
    let mut r = reqwest::blocking::get(COMMUNITY_SERVERS_LIST_URL)?;

    let mut buf: String = String::new();
    r.read_to_string(&mut buf)?;

    Ok(serde_xml_rs::from_str::<CommunityServers>(&buf)?)
}
