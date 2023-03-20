use std::time::Duration;

use crate::LinkResolver;
use anyhow::Result;
use async_trait::async_trait;

pub struct HTTPSLinkResolver {
    http_client: reqwest::Client,
}

impl HTTPSLinkResolver {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::builder().use_rustls_tls().build()?,
        })
    }
}

#[async_trait]
impl LinkResolver for HTTPSLinkResolver {
    async fn download(&self, uri: &str) -> Result<Vec<u8>> {
        let content = self
            .http_client
            .get(uri)
            .timeout(Duration::from_secs(5))
            .send()
            .await?
            .text()
            .await?;
        debug!("downloaded {}", content);
        Ok(content.as_bytes().to_vec())
        // todo: pass string
    }
}