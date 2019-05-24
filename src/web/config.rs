#[cfg(feature = "cli")]
use structopt::StructOpt;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub enum Config {
    #[cfg_attr(feature = "cli", structopt(name = "mime-types"))]
    MimeTypes,
    #[cfg_attr(feature = "cli", structopt(name = "detectors"))]
    Detectors,
    #[cfg_attr(feature = "cli", structopt(name = "parsers"))]
    Parsers,
    #[cfg_attr(feature = "cli", structopt(name = "parsers-details"))]
    ParsersDetails,
}

impl Config {
    pub fn path(&self) -> &'static str {
        match self {
            Config::MimeTypes => "mime-types",
            Config::Detectors => "detectors",
            Config::Parsers => "parsers",
            Config::ParsersDetails => "parsers/details",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub endpoint: String,
    pub produces: Vec<MimeType>,
    pub jvm_class: String,
    pub jvm_method: String,
    pub http_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimeType {
    pub identifier: String,
    pub supertype: Option<String>,
    pub alias: Vec<String>,
    pub parser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MimeTypeInner {
    pub supertype: Option<String>,
    pub alias: Vec<String>,
    pub parser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parser {
    #[serde(default)]
    pub children: Vec<Parser>,
    pub composite: bool,
    /// the name of the parser's jvm class
    pub name: String,
    pub decorated: bool,
    #[serde(rename = "supportedTypes", default)]
    pub supported_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detector {
    pub name: String,
    pub composite: bool,
    #[serde(default)]
    pub children: Vec<Detector>,
}
