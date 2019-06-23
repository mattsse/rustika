use crate::error::{Error, Result};
use crate::web::config::{Config, Detector, MimeType, MimeTypeInner, Parser};
use crate::web::translate::{Language, Translator};
use crate::TikaMode;
use reqwest::{self, Body, IntoUrl, Request, Response, Url};
use std::io::{BufRead, BufReader};
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

/// The client to interact with a tika server
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
    pub fn start_server(&mut self) -> Result<()> {
        if let TikaMode::ClientServer(addr) = self.config.tika_mode {
            let server_file = match self.config.tika_server_file {
                TikaServerFileLocation::Remote(_) => self.download_server_jar()?,
                TikaServerFileLocation::File(ref file) => file,
            };

            let mut handle = server_file.start_server(&addr)?;

            let stderr = handle
                .stderr
                .as_mut()
                .ok_or(Error::config("Failed to read tika server logs"))?;

            let reader = BufReader::new(stderr);

            // wait until the tika server is launched, which is indicated by a log message
            for line in reader.lines() {
                let line = line?;
                if self.config.server_verbosity == Verbosity::Verbose {
                    println!("{}", line);
                }
                if line.starts_with("INFO  Started Apache Tika server at") {
                    break;
                }
            }

            // remove output pipes if verbose logging is configured
            if self.config.server_verbosity == Verbosity::Verbose {
                handle.stderr.take();
                handle.stdout.take();
            }

            self.server_handle = Some(handle);
            Ok(())
        } else {
            Err(Error::config(
                "Client is configured as `ClientOnly` and can't spawn a server instance,",
            ))
        }
    }

    /// restart the server and use a a different local address, if supplied
    pub fn restart_server(&mut self, addr: Option<SocketAddr>) -> Result<()> {
        let _ = self.stop_server()?;
        if let Some(addr) = addr {
            self.config.tika_mode = TikaMode::ClientServer(addr);
            self.server_endpoint = self.config.tika_mode.server_endpoint();
        }
        self.start_server()
    }

    /// Shuts down the spawned tika server
    pub fn stop_server(&mut self) -> Result<()> {
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

    /// downloads the tika server jar
    pub(crate) fn download_server_jar(&mut self) -> Result<&TikaServerFile> {
        debug!("Fetching tika server jar file.");
        let mut resp = self
            .client
            .get(&TikaConfig::remote_server_jar(&self.config.tika_version))
            .send()?;
        let server_jar = self.config.tika_path.join("tika-server.jar");

        let mut out = fs::File::create(&server_jar)?;
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

    /// the endpoint of the tika server
    pub fn server_endpoint(&self) -> &Url {
        &self.server_endpoint
    }

    pub fn is_server_live(&self) -> bool {
        unimplemented!()
    }

    /// Joins the configured tika server endpoint with the `path`
    #[inline]
    pub fn endpoint_url<T: AsRef<str>>(&self, path: T) -> Result<Url> {
        Ok(self.server_endpoint.join(path.as_ref())?)
    }

    #[inline]
    pub fn request(&self, request: Request) -> Result<Response> {
        Ok(self.client.execute(request)?)
    }

    /// sends a GET request to the `tika_url` with the `Accept` header set to `application/json`
    #[inline]
    pub fn get_json(&self, path: &str) -> Result<Response> {
        Ok(self
            .client
            .get(self.endpoint_url(path)?)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()?)
    }

    /// Returns all the configured `Detector` of the tika server
    /// A `Detector` can contain child `Detectors`.
    /// Therefor the tika server returns a single `Detector` at this endpoint
    /// (the `DefaultDetector`) from which all other `Detectors` inherit.
    pub fn detectors(&self) -> Result<Detector> {
        Ok(serde_json::from_reader(
            self.get_json(Config::Detectors.path())?,
        )?)
    }

    /// Returns all the configured `Parser` of the tika server
    /// A `Parser` can contain child `Parser`.
    /// Therefor the tika server returns a single `Parser` at this endpoint
    /// (the `DefaultParser`) from which all other `Parser` inherit.
    pub fn parsers(&self) -> Result<Parser> {
        Ok(serde_json::from_reader(
            self.get_json(Config::Parsers.path())?,
        )?)
    }

    /// Returns the configured `Parser` with additional information.
    pub fn parsers_details(&self) -> Result<Parser> {
        Ok(serde_json::from_reader(
            self.get_json(Config::ParsersDetails.path())?,
        )?)
    }

    /// returns all the mime types configured on the server
    pub fn mime_types(&self) -> Result<Vec<MimeType>> {
        let resp = self.get_json(Config::MimeTypes.path())?;

        let mimes: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_reader(resp)?;

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

    ///  Translates the content of to destination language by auto detecting the source language using the configured translator
    pub fn translate_auto<T: Into<Body>, D: Into<Language>>(
        &self,
        content: T,
        dest_lang: D,
    ) -> Result<String> {
        self.put_translate(
            content,
            None,
            dest_lang.into(),
            &self.config.tika_translator,
        )
    }

    ///  Translates the content of source file from src language to destination language using the configured translator
    pub fn translate<T: Into<Body>, S: Into<Language>, D: Into<Language>>(
        &self,
        content: T,
        src_lang: S,
        dest_lang: D,
    ) -> Result<String> {
        self.put_translate(
            content,
            Some(src_lang.into()),
            dest_lang.into(),
            &self.config.tika_translator,
        )
    }
    ///  Translates the content of source file from src language to destination language
    /// using a specific translator
    pub fn translate_with_translator<T: Into<Body>, S: Into<Language>, D: Into<Language>>(
        &self,
        content: T,
        src_lang: S,
        dest_lang: D,
        translator: &Translator,
    ) -> Result<String> {
        self.put_translate(content, Some(src_lang.into()), dest_lang.into(), translator)
    }

    ///  Translates the content of source file to destination language by auto detecting the source language
    /// using a specific translator
    pub fn translate_with_translator_auto<T: Into<Body>, S: Into<Language>, D: Into<Language>>(
        &self,
        content: T,
        dest_lang: D,
        translator: &Translator,
    ) -> Result<String> {
        self.put_translate(content, None, dest_lang.into(), translator)
    }

    fn put_translate<T: Into<Body>>(
        &self,
        content: T,
        src_lang: Option<Language>,
        dest_lang: Language,
        translator: &Translator,
    ) -> Result<String> {
        let mut path = format!("translate/all/{}/", translator.as_str());
        if let Some(src_lang) = src_lang {
            path = format!("{}{}/", path, src_lang.0);
        }
        path += &dest_lang.0;

        let mut resp = self
            .client
            .put(self.endpoint_url(path)?)
            .header(reqwest::header::ACCEPT, "text/plain")
            .body(content.into())
            .send()?;
        Ok(resp.text()?)
    }

    /// Detects MIME type of the content.
    /// The resulting mime type will only include the `identifier` field
    /// A empty body will result in a `application/octet-stream` mime type.
    ///
    /// # Example
    ///
    /// Detect the mime type of a file
    ///
    /// ```edition2018
    /// # use rustika::TikaClient;
    /// # fn run() -> rustika::Result<()> {
    /// let client = TikaClient::default();
    /// let mime_type = client.detect_mime(::std::fs::read("Cargo.toml")?)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_mime<T: Into<Body>>(&self, content: T) -> Result<MimeType> {
        let mut resp = self
            .client
            .put(self.endpoint_url("detect/stream")?)
            .header(reqwest::header::ACCEPT, "text/plain")
            .body(content.into())
            .send()?;
        Ok(MimeType::new(resp.text()?))
    }

    /// Detects the language of the content
    /// A empty body will result in a empty response that is treated as an error.
    ///
    /// # Example
    ///
    /// Detect the language type of a file
    ///
    /// ```edition2018
    /// # use rustika::TikaClient;
    /// # fn run() -> rustika::Result<()> {
    /// let client = TikaClient::default();
    /// let mime_type = client.detect_language(::std::fs::read("Cargo.toml")?)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_language<T: Into<Body>>(&self, content: T) -> Result<Language> {
        let mut resp = self
            .client
            .put(self.endpoint_url("language/stream")?)
            .header(reqwest::header::ACCEPT, "text/plain")
            .body(content.into())
            .send()?;
        let lang = resp.text()?;
        if lang.is_empty() {
            Err(Error::server(
                "Failed to detect language. Got empty response.",
            ))
        } else {
            Ok(lang.into())
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
        // kill the spawned server
        self.stop_server().expect(&format!(
            "Failed shutting down the Tika Server on {}",
            self.server_endpoint
        ));
    }
}

/// Builder struct to create a `TikaClient`
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
    pub tika_translator: Option<Translator>,
    /// whether the tika server should log to std::out
    pub server_verbosity: Verbosity,
}

impl TikaBuilder {
    /// Creates a new builder for the desired `tika_mode`
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

    /// Constructs a new `TikaBuilder` in Client mode, targeting the `server_url`
    ///
    /// # Example
    ///
    /// Redirect any call to running tika server on localhost
    /// ```edition2018
    /// # fn main() -> rustika::Result<()> {
    /// let client = rustika::TikaBuilder::client_only("http://localhost:9998")?.build();
    /// # Ok(())
    /// # }
    /// ```
    /// Target a remote tika server
    ///
    /// ```edition2018
    /// # fn main() -> rustika::Result<()> {
    /// let client = rustika::TikaBuilder::client_only("https://example-tika.org")?.build();
    /// # Ok(())
    ///  # }
    /// ```
    pub fn client_only<U: IntoUrl>(server_url: U) -> Result<Self> {
        Ok(TikaBuilder::new(TikaMode::client_only(server_url)?))
    }

    /// Constructs a new `TikaBuilder` in ClientServer mode, to spawn a new tika server instance at `addr`
    pub fn with_server<T: AsRef<str>>(addr: T) -> Result<Self> {
        Ok(TikaBuilder::new(TikaMode::client_server(addr)?))
    }

    /// The version of the tika server to download if no `TIKA_SERVER_JAR` is set.
    /// Can be set with `TIKA_VERSION`
    pub fn version<T: Into<String>>(mut self, version: T) -> Self {
        self.tika_version = Some(version.into());
        self
    }

    /// The path where tika files should be stored.
    /// This will be a tempfile if no `TIKA_PATH` is set
    pub fn path<T: AsRef<Path>>(mut self, path: T) -> Self {
        self.tika_path = Some(path.as_ref().into());
        self
    }

    /// The location of the tika server file.
    /// If no `TIKA_SERVER_JAR` is or the `tika-rest-server` executable is not on `PATH`,
    /// this will be a pointer to the remote download link of the tika server jar file.
    pub fn server_file(mut self, server_file: TikaServerFileLocation) -> Self {
        self.tika_server_file = server_file;
        self
    }

    /// The specific tika translator class to use translate docs on the server.
    /// By default the `org.apache.tika.language.translate.Lingo24Translator` class is configured
    ///
    pub fn translator<T: Into<Translator>>(mut self, translator: T) -> Self {
        self.tika_translator = Some(translator.into());
        self
    }

    /// How a spawned tika server should log.
    /// By default the server will be `Verbosity::Silent` and not log to `std::out` and `std::err`.
    pub fn server_verbosity(mut self, server_verbosity: Verbosity) -> Self {
        self.server_verbosity = server_verbosity;
        self
    }

    /// creates a new `TikaClient` and starts the server
    /// if no server file is available, it downloads it first
    pub fn start_server(self) -> Result<TikaClient> {
        let mut client = self.build();
        client.start_server()?;
        Ok(client)
    }

    /// Constructs a new `TikaClient` based on its configuration
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

/// How a spawned tika server should log to `std::out` and `std::err`
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Verbosity {
    /// don't log to current shell, use `Stdio::piped()` instead
    Silent,
    /// enable logging and print the tika server logs to `std::out`
    Verbose,
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Silent
    }
}

/// All configs of the `TikaClient`
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
    pub tika_translator: Translator,
    /// whether the tika server should log to std::out
    pub server_verbosity: Verbosity,
}

impl TikaConfig {
    /// The version of tika server to download if required
    #[inline]
    pub(crate) fn default_version() -> String {
        env::var("TIKA_VERSION").unwrap_or("1.20".to_string())
    }

    /// The specific translator class the tika server should use to translate docs
    #[inline]
    pub(crate) fn default_translator() -> Translator {
        env::var("TIKA_TRANSLATOR")
            .map(|x| Translator::Other(x))
            .unwrap_or(Translator::default())
    }

    /// The endpoint from which the tika server jar can be downloaded
    #[inline]
    pub(crate) fn remote_server_jar(version: &str) -> String {
        format!("http://search.maven.org/remotecontent?filepath=org/apache/tika/tika-server/{}/tika-server-{}.jar", version, version)
    }
}

impl Default for TikaConfig {
    fn default() -> Self {
        let tika_version = Self::default_version();

        TikaConfig {
            tika_version,
            tika_server_file: TikaServerFileLocation::default(),
            tika_path: env::temp_dir(),
            tika_mode: TikaMode::default(),
            tika_translator: Self::default_translator(),
            server_verbosity: Verbosity::Silent,
        }
    }
}

/// The location of the tika server jar/exe
/// Either a local jar or executable or a remote endpoint from which the server jar can be downloaded.
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

        // find the `tika-rest-server` executable from `Path`
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
    pub(crate) fn start_server(&self, addr: &SocketAddr) -> Result<Child> {
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
            .arg(addr.port().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Spawning {:?}", cmd);

        Ok(cmd.spawn()?)
    }
}
