use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unkown app id: {0}")]
    UnknownAppId(String),
    #[error("Failed to parse content: {0}")]
    FormatError(String),
}
