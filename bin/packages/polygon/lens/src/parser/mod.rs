use serde::{Deserialize, Serialize};
use serde_json;
mod error;
mod lenster;
mod phaver;

use crate::pb::{OffchainDataContent, RecordChanges};
use error::ParseError;

#[allow(dead_code)]
pub enum State {
    Queued,
    Completed,
    TimedOut,
    DownloadingFailed,
    ParsingFailed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PostMedia {
    item: String,
    r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct PostAttribute {
    traitType: String,
    displayType: String,
    value: String,
}

pub fn parse_content(content: &OffchainDataContent) -> Result<RecordChanges, ParseError> {
    let json: serde_json::Value = serde_json::from_str(&content.content)
        .map_err(|e| ParseError::FormatError(e.to_string()))?;

    let app_id = json
        .get("appId")
        .ok_or(ParseError::FormatError("Failed to find appId".to_string()))?
        .as_str()
        .ok_or(ParseError::FormatError("Failed to find appId".to_string()))?
        .to_string();

    match app_id.as_str() {
        "Lenster" => lenster::parse(content),
        "phaver" => phaver::parse(content),
        _ => Err(ParseError::UnknownAppId(app_id)),
    }
}
