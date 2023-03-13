use serde::{Deserialize, Serialize};

use super::error::ParseError;
use super::{PostAttribute, PostMedia, State};
use crate::pb::{
    record_change::Operation, value::Typed, Field, OffchainDataContent, RecordChange,
    RecordChanges, Value,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Post {
    appId: String,
    version: String,
    metadata_id: String,
    name: String,
    content: String,
    attributes: Vec<PostAttribute>,
    media: Vec<PostMedia>,
    image: Option<String>,
    imageMimeType: Option<String>,
    external_url: String,
    tags: Vec<String>,
    animation_url: Option<String>,
    mainContentFocus: String,
    contentWarning: Option<String>,
    locale: String,
}

pub fn parse(content: &OffchainDataContent) -> Result<RecordChanges, ParseError> {
    let data: Post = serde_json::from_str(&content.content)
        .map_err(|e| ParseError::FormatError(e.to_string()))?;

    Ok(RecordChanges {
        record_changes: vec![RecordChange {
            record: "lens_posts_offchain".to_string(),
            id: content.id.clone(),
            ordinal: 0,
            operation: Operation::Create.into(),
            fields: vec![
                Field {
                    name: "app_id".to_string(),
                    new_value: Some(Value {
                        typed: Some(Typed::String(data.appId)),
                    }),
                    old_value: None,
                },
                Field {
                    name: "name".to_string(),
                    new_value: Some(Value {
                        typed: Some(Typed::String(data.name)),
                    }),
                    old_value: None,
                },
                Field {
                    name: "content".to_string(),
                    new_value: Some(Value {
                        typed: Some(Typed::String(data.content)),
                    }),
                    old_value: None,
                },
                Field {
                    name: "state".to_string(),
                    new_value: Some(Value {
                        typed: Some(Typed::Uint32(1)),
                    }),
                    old_value: None,
                },
                Field {
                    name: "state".to_string(),
                    new_value: Some(Value {
                        typed: Some(Typed::Uint32(State::Completed as u32)),
                    }),
                    old_value: None,
                },
                // todo: add fields
            ],
        }],
    })
}
