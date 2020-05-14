use crate::models::WeeklyBasketOffer;
use crate::xlsx::{import_xlsx, ImportError};
use chrono::Utc;
use reqwest::Response;

use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("converting error")]
    ConvertingError(#[from] ImportError),
    #[error("network error")]
    NetworkError(#[from] reqwest::Error),
}
struct CrawlContent {
    response: Response,
    etag: Option<String>,
}
async fn get_page_content() -> Result<CrawlContent, CrawlError> {
    // We first check if there is new content
    let response = reqwest::get("https://www.fermedestilleuls.alsace/").await?;

    let etag = match response.headers().get("etag").map(|x| x.to_str()) {
        Some(Ok(e)) => Some(e.to_string()),
        _ => None,
    };

    Ok(CrawlContent { response, etag })
}

async fn week_offer(content: CrawlContent) -> Result<WeeklyBasketOffer, CrawlError> {
    let body = content.response.text().await?;

    let cursor = Cursor::new(body);
    let ok = import_xlsx(cursor)?;

    Ok(ok)
}

fn get_link_from_page(text: &str) -> Option<String> {
    let document = Document::from(text);

    let candidates: Vec<&str> = document
        .find(Name("a"))
        .flat_map(|x| x.attr("href"))
        .filter(|x| x.ends_with(".xlsx"))
        .collect();

    candidates.last().map(|x| x.to_string())
}

#[cfg(test)]
mod tests {
    use super::{get_link_from_page, get_page_content};

    #[test]
    fn crawl() {
        get_link_from_page(include_str!("../assets/page.html"));
    }

    #[tokio::test]
    async fn download() {
        let value = get_page_content().await;
    }
}
