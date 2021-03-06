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
    #[error("no link found")]
    NoDataFound,
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

async fn week_offer_from_response(response: Response) -> Result<WeeklyBasketOffer, CrawlError> {
    let body = response.bytes().await?;
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

pub async fn retrieve_new_xlsx(
    previous_etag: Option<&str>,
) -> Result<Option<WeeklyBasketOffer>, CrawlError> {
    //
    let content = get_page_content().await?;

    if previous_etag == content.etag.as_deref() {
        Ok(None)
    } else {
        // Retrieve the content of the HTML page
        let body = content.response.text().await?;

        if let Some(link) = get_link_from_page(&body) {
            println!("Link: {}", link);
            let xlsx_response = reqwest::get(&link).await?;
            let element = week_offer_from_response(xlsx_response).await?;

            Ok(Some(element))
        } else {
            Err(CrawlError::NoDataFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::get_link_from_page;

    #[test]
    fn crawl() {
        let link = get_link_from_page(include_str!("../tests_assets/page.html"))
            .expect("Shopuld have a valid link");
        let exp = "https://2cfd4adc-0de2-4679-a584-9d506c845b7a.filesusr.com/ugd/028d6a_765bb7d24a694e00bf5eebd67d3d6af4.xlsx?dn=Bon%20de%20commande%20-%20maj%20du%2011%20mai.xlsx";
        assert_eq!(exp, link);
    }
}
