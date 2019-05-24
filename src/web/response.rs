use crate::web::config::{Detector, MimeType, Parser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerConfig {
    Detectors(Vec<Detector>),
    Parsers(Vec<Parser>),
    MimeTypes(Vec<MimeType>),
    Endpoints,
}
