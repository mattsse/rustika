use crate::error::{Error, Result};
use crate::web::config::Config;
use crate::web::config::{Detector, MimeType, MimeTypeInner, Parser};
use crate::TikaMode;
use reqwest::{self, IntoUrl, Request, Response, Url};
use serde::export::Option::Some;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct ServerPolicy {
    addr: Option<SocketAddr>,
    download_missing_jar: bool,
}

impl Default for ServerPolicy {
    fn default() -> Self {
        ServerPolicy {
            addr: None,
            download_missing_jar: true,
        }
    }
}

#[derive(Debug)]
pub struct TikaClient {
    /// configuration of the tika server
    config: TikaConfig,
    /// endpoint of the tika server
    server_endpoint: Url,
    /// inner client to execute http requests
    pub(crate) client: reqwest::Client,
    /// handle to the spawned tika server
    pub server_handle: Option<Child>,
}

impl TikaClient {
    /// starts a local server instance
    pub fn start_server(&mut self, server_policy: ServerPolicy) {}

    pub fn stop_server(&mut self) {}

    pub fn restart_server(&mut self, server_policy: ServerPolicy) {}

    /// the endpoint of the tika server
    pub fn server_endpoint(&self) -> &Url {
        &self.server_endpoint
    }

    pub fn is_server_live(&self) -> bool {
        unimplemented!()
    }

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

    /// returns all the configured detectors of the tika server
    /// A `Detector` can contain child `Detectors`.
    /// Therefor the tika server returns a single `Detector` at this endpoint
    /// (the `DefaultDetector`) from which all other `Detectors` inherit.
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

    /// downloads the tika server jar
    pub fn download_server_jar(&mut self) -> Result<PathBuf> {
        debug!("Fetching tika server jar file.");
        let mut resp = self
            .client
            .get(&TikaConfig::remote_server_jar(&self.config.tika_version))
            .send()?;
        let server_jar = self.config.tika_path.join("tika-server.jar");

        if cfg!(feature = "cli") {
            // TODO add cli print loop for feature cli

        }

        let mut out = fs::File::create(&server_jar.clone())?;
        debug!("Downloading tika server jar to {}", server_jar.display());
        let written = std::io::copy(&mut resp, &mut out)?;

        debug!(
            "Finished download to {} with size {}.",
            server_jar.display(),
            written
        );

        self.config.tika_server = Some(TikaServerJar::Download(server_jar.clone()));

        Ok(server_jar)
    }
}

impl Default for TikaClient {
    fn default() -> Self {
        TikaBuilder::default().build()
    }
}

impl Drop for TikaClient {
    fn drop(&mut self) {
        // kill the spawned server
        if let Some(child) = &mut self.server_handle {
            match child.kill() {
                Err(e) => error!("Failed to shutdown the running tika server instance. {}", e),
                _ => debug!("Shutdown tika server"),
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TikaBuilder {
    /// how the the tika server is configured
    pub tika_mode: TikaMode,
    /// the version of tika
    pub tika_version: Option<String>,
    /// the path where to store installation and files
    pub tika_path: Option<PathBuf>,
    /// path to the tika server jar file
    pub tika_server_jar: Option<String>,
    /// translator class used to translate docs
    pub tika_translator: Option<String>,
    /// whether the tika server should log to std::out
    pub server_verbosity: Verbosity,
}

impl TikaBuilder {
    pub fn new(tika_mode: TikaMode) -> Self {
        TikaBuilder {
            tika_mode,
            tika_version: None,
            tika_path: None,
            tika_server_jar: None,
            tika_translator: None,
            server_verbosity: Verbosity::default(),
        }
    }

    pub fn client_only<U: IntoUrl>(server_url: U) -> Result<Self> {
        Ok(TikaBuilder::new(TikaMode::ClientOnly(
            server_url.into_url()?,
        )))
    }

    pub fn with_server<T: AsRef<str>>(addr: T) -> Result<Self> {
        Ok(TikaBuilder::new(TikaMode::client_server(addr)?))
    }

    pub fn version<T: Into<String>>(mut self, version: T) -> Self {
        self.tika_version = Some(version.into());
        self
    }

    pub fn path<T: AsRef<Path>>(mut self, path: T) -> Self {
        self.tika_path = Some(path.as_ref().into());
        self
    }

    pub fn server_jar<T: Into<String>>(mut self, server_jar: T) -> Self {
        self.tika_server_jar = Some(server_jar.into());
        self
    }

    pub fn translator<T: Into<String>>(mut self, server_jar: T) -> Self {
        self.tika_server_jar = Some(server_jar.into());
        self
    }
    pub fn server_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.server_verbosity = verbosity;
        self
    }

    pub fn build(self) -> TikaClient {
        let tika_version = self.tika_version.unwrap_or(TikaConfig::default_version());
        let config = TikaConfig {
            tika_server: None,
            tika_version,
            tika_path: self.tika_path.unwrap_or(env::temp_dir()),
            tika_mode: self.tika_mode,
            tika_translator: self
                .tika_translator
                .unwrap_or(TikaConfig::default_translator()),
            server_verbosity: self.server_verbosity,
        };

        TikaClient {
            client: reqwest::Client::new(),
            server_endpoint: config.tika_mode.server_endpoint(),
            server_handle: None,
            config,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Verbosity {
    /// don't log to current shell
    Silent,
    /// enable logging
    Verbose,
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Silent
    }
}

#[derive(Debug, Clone)]
pub struct TikaConfig {
    /// the version of tika
    pub tika_version: String,
    /// the path where to store installation and files
    pub tika_path: PathBuf,
    /// path to the tika server jar
    pub tika_server: Option<TikaServerJar>,
    /// how the the tika server is configured
    pub tika_mode: TikaMode,
    /// translator class used to translate docs
    pub tika_translator: String,
    /// whether the tika server should log to std::out
    pub server_verbosity: Verbosity,
}

impl TikaConfig {
    #[inline]
    pub(crate) fn default_version() -> String {
        env::var("TIKA_VERSION").unwrap_or("1.20".to_string())
    }

    #[inline]
    pub(crate) fn default_translator() -> String {
        env::var("TIKA_TRANSLATOR")
            .unwrap_or("org.apache.tika.language.translate.Lingo24Translator".to_string())
    }

    #[inline]
    pub(crate) fn remote_server_jar(version: &str) -> String {
        format!("http://search.maven.org/remotecontent?filepath=org/apache/tika/tika-server/{}/tika-server-{}.jar", version, version)
    }
}

impl Default for TikaConfig {
    fn default() -> Self {
        let tika_version = Self::default_version();

        TikaConfig {
            tika_server: None,
            tika_version,
            tika_path: env::temp_dir(),
            tika_mode: TikaMode::default(),
            tika_translator: Self::default_translator(),
            server_verbosity: Verbosity::Silent,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TikaServer {
    File(TikaServerJar),
    Remote(String),
}

impl Default for TikaServer {
    fn default() -> Self {
        unimplemented!()
    }
}

/// points to a local location of the tika server
/// This can be
/// an `Os` executable:
/// (e.g. the homebrew installation of tika includes the system shell script `tika-rest-server` )
/// or a jar file, which is either downloaded or pointed to by the `TIKA_SERVER_JAR` env variable
#[derive(Debug, Clone)]
pub enum TikaServerJar {
    /// `tika-rest-server`executable directly in `PATH`
    Os(PathBuf),
    /// env var pointer to a tika server jar
    Var(PathBuf),
    /// stores the path to a downloaded server jar either in `TIKA_PATH` or within a temp dir
    Download(PathBuf),
}

impl TikaServerJar {
    /// returns a env var pointer, either `TikaServer::Os` or `TikaServer::Var` to a tika server file.
    /// `TikaServer::Var` trumps any OS variable
    pub fn from_env() -> Result<Self> {
        if let Ok(path) = env::var("TIKA_SERVER_JAR").map(PathBuf::from) {
            return if path.exists() {
                Ok(TikaServerJar::Var(path))
            } else {
                Err(Error::config(format!(
                    "Set `TIKA_SERVER_JAR` at {} does not exist",
                    path.display()
                )))
            };
        }

        Err(Error::config("Failed to retrieve"))
    }

    /// checks whether the file it points to exists
    pub fn exists(&self) -> bool {
        self.location().exists()
    }

    /// the location of the server executable or jar file
    pub fn location(&self) -> &PathBuf {
        match self {
            TikaServerJar::Os(path) | TikaServerJar::Var(path) | TikaServerJar::Download(path) => {
                path
            }
        }
    }

    /// starts a new server instance and returns the handle to the spawned process
    pub fn start_server(&self, addr: &SocketAddr, server_verbosity: Verbosity) -> Result<Child> {
        debug!("launching tika server from {}", self.location().display());
        let mut cmd = match self {
            TikaServerJar::Os(path) => Command::new(path),
            TikaServerJar::Var(path) | TikaServerJar::Download(path) => {
                let java_path = which::which("java").expect("Failed to locate java in PATH.");
                let mut cmd = Command::new(java_path);
                cmd.arg("-cp")
                    .arg(path)
                    .arg("org.apache.tika.server.TikaServerCli");
                cmd
            }
        };

        cmd.arg("--host")
            .arg(addr.ip().to_string())
            .arg("--port")
            .arg(addr.port().to_string());

        debug!("Spawning {:?}", cmd);

        if server_verbosity == Verbosity::Silent {
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        Ok(cmd.spawn()?)
    }
}
