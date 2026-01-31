//! HTTP client implementation using reqwest.

use std::path::Path;

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::core::infra::HttpClient;
use crate::core::types::{Error, Result};

/// HTTP client implementation using reqwest.
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    #[allow(dead_code)] // Used in trait implementation methods
    client: reqwest::Client,
}

impl ReqwestClient {
    /// Create a new ReqwestClient instance.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn get(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))
    }

    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| Error::Network(e.to_string()))
    }

    async fn download(&self, url: &str, dest: &Path) -> Result<()> {
        let bytes = self.get_bytes(url).await?;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = File::create(dest).await?;
        file.write_all(&bytes).await?;
        Ok(())
    }
}
