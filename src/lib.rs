use reqwest::blocking::Client;
use select::{document::Document, predicate::Name};
use thiserror::Error;
use url::Url;

pub struct LinkExtractor {
    client: Client,
}

#[derive(Error, Debug)]
pub enum GetLinksError {
    #[error("failed to send a request")]
    SendRequest(#[source] reqwest::Error),
    #[error("failed to read the response bodys")]
    ResponseBody(#[source] reqwest::Error),
    #[error("failed to make the link url absolute")]
    AbsolutizeUrl(#[source] url::ParseError),
    #[error("server returned an error")]
    ServerError(#[source] url::ParseError),
}

impl LinkExtractor {
    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub fn get_links(&self, url: Url) -> Result<Vec<Url>, eyre::Report> {
        log::info!("GET {}", url);
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| GetLinksError::SendRequest(e))?;
        let base_url = response.url().clone();
        let status = response.status();
        let body = response.text()?;
        let doc = Document::from(body.as_str());
        let mut links = Vec::new();
        log::info!("Retrieved {}, {}", status, base_url);

        for href in doc.find(Name("a")).filter_map(|a| a.attr("href")) {
            match Url::parse(href) {
                Ok(mut url) => {
                    url.set_fragment(None);
                    links.push(url);
                }
                Err(url::ParseError::RelativeUrlWithoutBase) => match base_url.join(href) {
                    Ok(mut url) => {
                        url.set_fragment(None);
                        links.push(url)
                    }
                    Err(e) => {
                        log::warn!("URL join error: {}", e)
                    }
                },
                Err(e) => {
                    println!("Error: {}", e)
                }
            }
        }

        Ok(links)
    }
}
