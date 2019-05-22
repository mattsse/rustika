use crate::error::{ErrorKind, Result};
use crate::web::{Detector, Endpoint, MimeType, MimeTypeInner};
use reqwest::{self, Client, Request, Response, Url};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TikaClient {
    /// the url to the tika-server
    pub server_url: String,
    info_cache: Option<Vec<Endpoint>>,

    client: Client,
}

impl TikaClient {
    /// returns all the endpoints defined in the url
    pub fn endpoints(&self) -> Result<Vec<Endpoint>> {
        Ok(vec![])
    }

    #[inline]
    pub(crate) fn endpoint_url(&self, path: &str) -> String {
        format!("{}/{}", self.server_url, path)
    }

    #[inline]
    pub fn request(&self, request: Request) -> Result<Response> {
        Ok(self.client.execute(request)?)
    }

    pub fn get_json(&self, tika_url: &str) -> Result<Response> {
        Ok(self
            .client
            .get(tika_url)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()?)
    }

    pub fn detectors(&self) -> Result<Vec<Detector>> {
        let resp = self.get_json(&self.endpoint_url("detectors"))?.text()?;
        Ok(serde_json::from_str(&resp)?)
    }

    pub fn mime_types(&self) -> Result<Vec<MimeType>> {
        let resp = self.get_json(&self.endpoint_url("detectors"))?;

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

#[derive(Debug, Clone)]
pub struct TikaBuilder {
    pub enable_info_cache: bool,
    pub url: String,
}

impl TikaBuilder {
    pub fn new(url: String) -> TikaBuilder {
        TikaBuilder {
            enable_info_cache: true,
            url,
        }
    }

    pub fn enable_info_cache(mut self, enable_info_cache: bool) -> Self {
        self.enable_info_cache = enable_info_cache;
        self
    }
}
