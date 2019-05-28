use crate::error::{Error, Result};
use crate::web::config::{Config, Detector, MimeType, MimeTypeInner, Parser};
use crate::TikaMode;
use reqwest::{self, IntoUrl, Request, Response, Url};
use serde::export::Option::Some;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{env, fs};

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
    /// handle to the spawned tika server
    server_handle: Option<Child>,
    /// inner client to execute http requests
    pub(crate) client: reqwest::Client,
}

impl TikaClient {
    /// starts a local server instance
    pub fn start_server(&mut self, server_policy: ServerPolicy) {}

    pub fn stop_server(&mut self) {}

    pub fn restart_server(&mut self, server_policy: ServerPolicy) {}

    fn kill_server(&mut self) -> Result<()> {
        // kill the spawned server
        if let Some(child) = &mut self.server_handle {
            match child.kill() {
                Err(e) => {
                    error!(
                        "Failed to shutdown the running tika server instance on {}: {}",
                        self.server_endpoint, e
                    );
                    Err(Error::server(format!(
                        "Failed to shutdown the running tika server instance on {}: {}",
                        self.server_endpoint, e
                    )))
                }
                _ => {
                    debug!("Shutdown tika server");
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// the endpoint of the tika server
    pub fn server_endpoint(&self) -> &Url {
        &self.server_endpoint
    }

    pub fn is_server_live(&self) -> bool {
        unimplemented!()
    }

    #[inline]
    pub fn endpoint_url<T: AsRef<str>>(&self, path: T) -> Result<Url> {
        Ok(self.server_endpoint.join(path.as_ref())?)
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

    /// returns all the mime types configured on the server
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
    pub fn download_server_jar(&mut self) -> Result<&TikaServerFile> {
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

        self.config.tika_server_file =
            TikaServerFileLocation::File(TikaServerFile::Download(server_jar));

        match &self.config.tika_server_file {
            TikaServerFileLocation::File(file) => Ok(file),
            _ => unreachable!(),
        }
    }
}

impl Default for TikaClient {
    fn default() -> Self {
        TikaBuilder::default().build()
    }
}

impl Drop for TikaClient {
    fn drop(&mut self) {
        self.kill_server().expect(&format!(
            "Failed shutting down the Tika Server on {}",
            self.server_endpoint
        ));
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
    pub tika_server_file: TikaServerFileLocation,
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
            tika_server_file: TikaServerFileLocation::default(),
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

    pub fn server_file(mut self, server_file: TikaServerFileLocation) -> Self {
        self.tika_server_file = server_file;
        self
    }

    pub fn translator<T: Into<String>>(mut self, translator: T) -> Self {
        self.tika_translator = Some(translator.into());
        self
    }

    pub fn server_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.server_verbosity = verbosity;
        self
    }

    /// creates a new `TikaClient` and starts the server
    /// if no server file is available, it downloads it first
    pub fn start_server(self) -> Result<TikaClient> {
        if let TikaMode::ClientServer(addr) = self.tika_mode {
            let mut client = self.build();
            let verbosity = client.config.server_verbosity;

            let server_file = match client.config.tika_server_file {
                TikaServerFileLocation::Remote(_) => client.download_server_jar()?,
                TikaServerFileLocation::File(ref file) => file,
            };

            let handle = server_file.start_server(&addr, verbosity)?;
            client.server_handle = Some(handle);
            Ok(client)
        } else {
            Err(Error::config(
                "Can not start tika server, because client is configured as client only.",
            ))
        }
    }

    pub fn build(self) -> TikaClient {
        let tika_version = self.tika_version.unwrap_or(TikaConfig::default_version());
        let config = TikaConfig {
            tika_server_file: TikaServerFileLocation::default(),
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
    pub tika_server_file: TikaServerFileLocation,
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
            tika_server_file: TikaServerFileLocation::default(),
            tika_version,
            tika_path: env::temp_dir(),
            tika_mode: TikaMode::default(),
            tika_translator: Self::default_translator(),
            server_verbosity: Verbosity::Silent,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TikaServerFileLocation {
    /// local jar or executable
    File(TikaServerFile),
    /// endpoint of the tika server jar
    Remote(String),
}

impl Default for TikaServerFileLocation {
    fn default() -> Self {
        if let Ok(file) = TikaServerFile::from_env() {
            TikaServerFileLocation::File(file)
        } else {
            TikaServerFileLocation::Remote(TikaConfig::remote_server_jar(
                &TikaConfig::default_version(),
            ))
        }
    }
}

/// points to a local location of the tika server
/// This can be
/// an `Os` executable:
/// (e.g. the homebrew installation of tika includes the system shell script `tika-rest-server` )
/// or a jar file, which is either downloaded or pointed to by the `TIKA_SERVER_JAR` env variable
#[derive(Debug, Clone)]
pub enum TikaServerFile {
    /// `tika-rest-server`executable directly in `PATH`
    PathExecutable(PathBuf),
    /// env var pointer to a tika server jar
    EnvVarJar(PathBuf),
    /// stores the path to a downloaded server jar either in `TIKA_PATH` or within a temp dir
    Download(PathBuf),
}

impl TikaServerFile {
    /// returns a env var pointer, either `TikaServer::Os` or `TikaServer::Var` to a tika server file.
    /// `TikaServer::Var` trumps any OS variable
    pub fn from_env() -> Result<Self> {
        match env::var("TIKA_SERVER_JAR").map(PathBuf::from) {
            Ok(path) => return Ok(TikaServerFile::EnvVarJar(path)),
            Err(env::VarError::NotUnicode(var)) => Err(()).expect(&format!(
                "TIKA_SERVER_JAR env var found but did not contain valid unicode {:?}",
                var
            )),
            _ => (),
        };

        match which::which("tika-rest-server") {
            Ok(path) => Ok(TikaServerFile::PathExecutable(path)),
            Err(_) => Err(Error::config(
                "Could not find system wide tika-rest-server executable",
            )),
        }
    }

    /// checks whether the file it points to exists
    pub fn exists(&self) -> bool {
        self.location().exists()
    }

    /// the location of the server executable or jar file
    pub fn location(&self) -> &PathBuf {
        match self {
            TikaServerFile::PathExecutable(path)
            | TikaServerFile::EnvVarJar(path)
            | TikaServerFile::Download(path) => path,
        }
    }

    /// starts a new server instance and returns the handle to the spawned process
    pub(crate) fn start_server(
        &self,
        addr: &SocketAddr,
        server_verbosity: Verbosity,
    ) -> Result<Child> {
        debug!("launching tika server from {}", self.location().display());
        let mut cmd = match self {
            TikaServerFile::PathExecutable(path) => Command::new(path),
            TikaServerFile::EnvVarJar(path) | TikaServerFile::Download(path) => {
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
