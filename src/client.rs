use crate::error::Result;
use crate::web::config::Config;
use crate::web::config::{Detector, MimeType, MimeTypeInner, Parser};
use crate::TikaMode;
use reqwest::{self, Client, Request, Response};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TikaClient {
    /// the url to the tika-server
    pub server_url: String,
    pub client: Client,
    mode: TikaMode,
}

impl TikaClient {
    #[inline]
    pub fn endpoint_url(&self, path: &str) -> String {
        format!("{}/{}", self.server_url, path)
    }

    #[inline]
    pub fn request(&self, request: Request) -> Result<Response> {
        Ok(self.client.execute(request)?)
    }

    /// sends a GET request to the `tika_url` with the `Accept` header set to `application/json`
    pub fn get_json(&self, tika_url: &str) -> Result<Response> {
        Ok(self
            .client
            .get(tika_url)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()?)
    }

    pub fn detectors(&self) -> Result<Detector> {
        Ok(serde_json::from_reader(
            self.get_json(&self.endpoint_url(Config::Detectors.path()))?,
        )?)
    }

    pub fn parsers(&self) -> Result<Parser> {
        Ok(serde_json::from_reader(
            self.get_json(&self.endpoint_url(Config::Parsers.path()))?,
        )?)
    }

    pub fn parsers_details(&self) -> Result<Parser> {
        Ok(serde_json::from_reader(self.get_json(
            &self.endpoint_url(Config::ParsersDetails.path()),
        )?)?)
    }

    pub fn mime_types(&self) -> Result<Vec<MimeType>> {
        let resp = self.get_json(&self.endpoint_url(Config::MimeTypes.path()))?;

        let mimes: HashMap<String, serde_json::Value> = serde_json::from_reader(resp)?;

        let mimes: ::std::result::Result<Vec<_>, _> = mimes
            .into_iter()
            .map(|(identifier, value)| {
                serde_json::from_value::<MimeTypeInner>(value).map(|x| MimeType {
                    identifier,
                    supertype: x.supertype,
                    alias: x.alias,
                    parser: x.parser,
                })
            })
            .collect();

        Ok(mimes?)
    }
}

impl Drop for TikaClient {
    fn drop(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct TikaBuilder {
    pub server_url: String,
    pub mode: TikaMode,
}

impl TikaBuilder {
    pub fn new(server_url: &str) -> TikaBuilder {
        TikaBuilder {
            server_url: server_url.to_string(),
            mode: TikaMode::default(),
        }
    }

    pub fn mode(mut self, mode: TikaMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn build(self) -> TikaClient {
        TikaClient {
            server_url: self.server_url,
            client: Client::new(),
            mode: self.mode,
        }
    }
}
