use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use tracing::error;

use crate::domain::{errors::DomainError, scheme::SchemeNews};

const RSS_URL: &str =
    "https://news.google.com/rss/search?q=government+schemes+farmers+yojana+India&hl=en-IN&gl=IN&ceid=IN:en";

const MAX_ITEMS: usize = 20;

static ITEM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<item>(.*?)</item>").unwrap());
static TITLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<title>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</title>").unwrap());
static LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<link>(.*?)</link>").unwrap());
static PUBDATE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<pubDate>(.*?)</pubDate>").unwrap());
static DESC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<description>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</description>").unwrap()
});
static SOURCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<source[^>]*>(.*?)</source>").unwrap());

#[derive(Clone)]
pub struct NewsFetcher {
    http: Client,
}

impl NewsFetcher {
    pub fn new() -> Self {
        Self { http: Client::new() }
    }

    pub async fn fetch_schemes_news(&self) -> Result<Vec<SchemeNews>, DomainError> {
        let body = self
            .http
            .get(RSS_URL)
            .header("User-Agent", "Mozilla/5.0 (compatible; KishanMitra/1.0)")
            .send()
            .await
            .map_err(|e| {
                error!("RSS fetch failed: {e}");
                DomainError::Internal("Failed to fetch latest scheme news".into())
            })?
            .text()
            .await
            .map_err(|e| {
                error!("RSS read failed: {e}");
                DomainError::Internal("Failed to read scheme news response".into())
            })?;

        Ok(parse_rss(&body))
    }
}

fn parse_rss(xml: &str) -> Vec<SchemeNews> {
    ITEM_RE
        .captures_iter(xml)
        .take(MAX_ITEMS)
        .filter_map(|cap| {
            let item = cap.get(1)?.as_str();

            let title = extract(item, &TITLE_RE)?;
            let url = extract(item, &LINK_RE)?;
            let published_at = extract(item, &PUBDATE_RE).unwrap_or_default();
            let description = extract(item, &DESC_RE).unwrap_or_default();
            let source = extract(item, &SOURCE_RE).unwrap_or_else(|| "Unknown".to_string());

            Some(SchemeNews {
                title: decode_entities(&title),
                url,
                source: decode_entities(&source),
                published_at,
                description: decode_entities(&description),
            })
        })
        .collect()
}

fn extract(text: &str, re: &Regex) -> Option<String> {
    re.captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
}
