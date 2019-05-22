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
    /// the name of the parser
    pub name: String,

    /// the class of the java parser class
    pub jvm_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserDetails {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detector {
    pub name: String,
    pub composite: bool,
    pub children: Vec<Detector>,
}
