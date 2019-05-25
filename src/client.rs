use crate::error::Result;
use crate::web::config::Config;
use crate::web::config::{Detector, MimeType, MimeTypeInner, Parser};
use crate::TikaMode;
use reqwest::{self, Client, Request, Response, Url};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub struct TikaClient {
    /// the url to the tika-server
    config: TikaConfig,
    server_endpoint: Url,
    pub client: Client,
    mode: TikaMode,
}

impl TikaClient {
    pub fn start_server(&mut self) {}

    pub fn stop_server(&mut self) {}

    pub fn restart_server(&mut self) {}

    #[inline]
    pub fn endpoint_url(&self, path: &str) -> Result<Url> {
        Ok(self.server_endpoint.join(path)?)
    }

    #[inline]
    pub fn request(&self, request: Request) -> Result<Response> {
        Ok(self.client.execute(request)?)
    }

    /// sends a GET request to the `tika_url` with the `Accept` header set to `application/json`
    pub fn get_json(&self, path: &str) -> Result<Response> {
        Ok(self
            .client
            .get(self.endpoint_url(path)?)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()?)
    }

    pub fn detectors(&self) -> Result<Detector> {
        Ok(serde_json::from_reader(
            self.get_json(Config::Detectors.path())?,
        )?)
    }

    pub fn parsers(&self) -> Result<Parser> {
        Ok(serde_json::from_reader(
            self.get_json(Config::Parsers.path())?,
        )?)
    }

    pub fn parsers_details(&self) -> Result<Parser> {
        Ok(serde_json::from_reader(
            self.get_json(Config::ParsersDetails.path())?,
        )?)
    }

    pub fn mime_types(&self) -> Result<Vec<MimeType>> {
        let resp = self.get_json(Config::MimeTypes.path())?;

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
    fn drop(&mut self) {
        // TODO stop running server
    }
}

#[derive(Debug, Clone, Default)]
pub struct TikaBuilder {
    /// how the the tika server is configured
    pub tika_mode: TikaMode,
    pub tika_version: Option<String>,
    /// the path where to store installation and files
    pub tika_path: Option<PathBuf>,
    /// path to the tika server jar file
    pub tika_server_jar: Option<String>,
    /// translator class used to translate docs
    pub tika_translator: Option<String>,
    /// whether the tika server should log to std::out
    pub verbose: bool,
}

impl TikaBuilder {
    pub fn new(tika_mode: TikaMode) -> TikaBuilder {
        TikaBuilder {
            tika_mode,
            tika_version: None,
            tika_path: None,
            tika_server_jar: None,
            tika_translator: None,
            verbose: false,
        }
    }

    pub fn client_only(self) -> Result<TikaClient> {
        unimplemented!()
    }

    pub fn with_server(self) -> Result<TikaClient> {
        unimplemented!()
    }

    pub fn build(self) -> TikaClient {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct TikaConfig {
    /// the version of tika
    pub tika_version: String,
    /// the path where to store installation and files
    pub tika_path: PathBuf,
    /// path to the tika server jar file
    pub tika_server_jar: String,
    /// how the the tika server is configured
    pub tika_mode: TikaMode,
    /// translator class used to translate docs
    pub tika_translator: String,
    /// whether the tika server should log to std::out
    pub verbose: bool,
}

impl TikaConfig {
    fn start_server(&self) -> Result<Child> {
        // if verbose: piped, otherwise inherit
        let mut cmd = Command::new("");
        if !self.verbose {
            cmd.stdout(Stdio::piped());
        }

        unimplemented!()
    }
}

impl Default for TikaConfig {
    fn default() -> Self {
        let tika_version = env::var("TIKA_VERSION").unwrap_or("1.20".to_string());

        TikaConfig {
            tika_server_jar: env::var("TIKA_SERVER_JAR").unwrap_or(format!("http://search.maven.org/remotecontent?filepath=org/apache/tika/tika-server/{}/tika-server-{}.jar", tika_version, tika_version)),
            tika_version,
            tika_path : env::temp_dir(),
            tika_mode: TikaMode::default(),
            tika_translator: env::var("TIKA_TRANSLATOR").unwrap_or("org.apache.tika.language.translate.Lingo24Translator".to_string()),
            verbose: true,
        }

        //    Url::parse(&env::var("TIKA_SERVER_ENDPOINT").unwrap_or("http://localhost:9998".to_string(),
        //    ))?,
    }
}
