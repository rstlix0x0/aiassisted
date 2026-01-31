//! GitHub Releases API client for checking and downloading CLI updates.

use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

use crate::core::infra::HttpClient;
use crate::core::selfupdate::ReleaseProvider;
use crate::core::types::{Error, ReleaseInfo, Result};

use super::platform::Platform;
use super::version;

const GITHUB_REPO: &str = "rstlix0x0/aiassisted";
const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub Release API response.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub Release Asset.
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// GitHub Releases provider for self-updates.
pub struct GithubReleasesProvider<H: HttpClient> {
    http: H,
    platform: Platform,
}

impl<H: HttpClient> GithubReleasesProvider<H> {
    /// Create a new GitHub Releases provider.
    pub fn new(http: H) -> Self {
        Self {
            http,
            platform: Platform::detect(),
        }
    }

    /// Get the latest release from GitHub API.
    async fn fetch_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!("{}/repos/{}/releases/latest", GITHUB_API_BASE, GITHUB_REPO);

        let response = self
            .http
            .get(&url)
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch latest release: {}", e)))?;

        serde_json::from_str(&response)
            .map_err(|e| Error::Parse(format!("Failed to parse GitHub release: {}", e)))
    }

    /// Find the asset matching the current platform.
    fn find_platform_asset<'a>(&self, release: &'a GitHubRelease) -> Result<&'a GitHubAsset> {
        let asset_name = self.platform.asset_name();

        release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "No asset found for platform {} (expected: {})",
                    format!("{}-{}", self.platform.os, self.platform.arch),
                    asset_name
                ))
            })
    }
}

#[async_trait]
impl<H: HttpClient> ReleaseProvider for GithubReleasesProvider<H> {
    async fn get_latest(&self) -> Result<ReleaseInfo> {
        let release = self.fetch_latest_release().await?;
        let asset = self.find_platform_asset(&release)?;

        Ok(ReleaseInfo {
            version: release.tag_name.clone(),
            download_url: asset.browser_download_url.clone(),
            checksum: None, // GitHub doesn't provide checksums in the API response
        })
    }

    async fn is_update_available(&self, current_version: &str) -> Result<bool> {
        let latest = self.get_latest().await?;
        Ok(version::is_newer_version(current_version, &latest.version))
    }

    async fn download_release(&self, release: &ReleaseInfo, dest: &Path) -> Result<()> {
        self.http
            .download(&release.download_url, dest)
            .await
            .map_err(|e| Error::Network(format!("Failed to download release: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::infra::HttpClient;
    use async_trait::async_trait;
    use mockall::mock;
    use std::path::PathBuf;

    mock! {
        pub HttpClient {}

        #[async_trait]
        impl HttpClient for HttpClient {
            async fn get(&self, url: &str) -> Result<String>;
            async fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;
            async fn download(&self, url: &str, dest: &Path) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_get_latest_success() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v1.2.3",
            "assets": [
                {
                    "name": "aiassisted-x86_64-unknown-linux-gnu.tar.gz",
                    "browser_download_url": "https://github.com/rstlix0x0/aiassisted/releases/download/v1.2.3/aiassisted-x86_64-unknown-linux-gnu.tar.gz"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .withf(|url| url.contains("/repos/rstlix0x0/aiassisted/releases/latest"))
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.get_latest().await.unwrap();

        assert_eq!(result.version, "v1.2.3");
        assert!(result.download_url.contains("v1.2.3"));
        assert!(result.checksum.is_none());
    }

    #[tokio::test]
    async fn test_get_latest_network_error() {
        let mut mock_http = MockHttpClient::new();

        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Err(Error::Network("Connection failed".to_string())));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.get_latest().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Network(_)));
    }

    #[tokio::test]
    async fn test_get_latest_parse_error() {
        let mut mock_http = MockHttpClient::new();

        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Ok("invalid json".to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.get_latest().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Parse(_)));
    }

    #[tokio::test]
    async fn test_find_platform_asset_not_found() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v1.2.3",
            "assets": [
                {
                    "name": "aiassisted-aarch64-apple-darwin.tar.gz",
                    "browser_download_url": "https://github.com/example/repo/releases/download/v1.2.3/binary-aarch64.tar.gz"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.get_latest().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[tokio::test]
    async fn test_is_update_available_newer() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v2.0.0",
            "assets": [
                {
                    "name": "aiassisted-x86_64-unknown-linux-gnu.tar.gz",
                    "browser_download_url": "https://github.com/example/repo/releases/download/v2.0.0/binary.tar.gz"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.is_update_available("v1.0.0").await.unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn test_is_update_available_same_version() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v1.0.0",
            "assets": [
                {
                    "name": "aiassisted-x86_64-unknown-linux-gnu.tar.gz",
                    "browser_download_url": "https://github.com/example/repo/releases/download/v1.0.0/binary.tar.gz"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.is_update_available("v1.0.0").await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_is_update_available_older() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v1.0.0",
            "assets": [
                {
                    "name": "aiassisted-x86_64-unknown-linux-gnu.tar.gz",
                    "browser_download_url": "https://github.com/example/repo/releases/download/v1.0.0/binary.tar.gz"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.is_update_available("v2.0.0").await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_download_release_success() {
        let mut mock_http = MockHttpClient::new();

        mock_http
            .expect_download()
            .withf(|url, _| url.contains("github.com"))
            .times(1)
            .returning(|_, _| Ok(()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let release = ReleaseInfo {
            version: "v1.2.3".to_string(),
            download_url: "https://github.com/example/repo/releases/download/v1.2.3/binary.tar.gz"
                .to_string(),
            checksum: None,
        };

        let dest = PathBuf::from("/tmp/binary.tar.gz");
        let result = provider.download_release(&release, &dest).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_release_network_error() {
        let mut mock_http = MockHttpClient::new();

        mock_http
            .expect_download()
            .times(1)
            .returning(|_, _| Err(Error::Network("Download failed".to_string())));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let release = ReleaseInfo {
            version: "v1.2.3".to_string(),
            download_url: "https://github.com/example/repo/releases/download/v1.2.3/binary.tar.gz"
                .to_string(),
            checksum: None,
        };

        let dest = PathBuf::from("/tmp/binary.tar.gz");
        let result = provider.download_release(&release, &dest).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Network(_)));
    }

    #[tokio::test]
    async fn test_macos_platform_asset() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v1.0.0",
            "assets": [
                {
                    "name": "aiassisted-aarch64-apple-darwin.tar.gz",
                    "browser_download_url": "https://github.com/example/repo/releases/download/v1.0.0/binary.tar.gz"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "macos".to_string(),
                arch: "aarch64".to_string(),
            },
        };

        let result = provider.get_latest().await.unwrap();

        assert_eq!(result.version, "v1.0.0");
    }

    #[tokio::test]
    async fn test_windows_platform_asset() {
        let mut mock_http = MockHttpClient::new();

        let response = r#"{
            "tag_name": "v1.0.0",
            "assets": [
                {
                    "name": "aiassisted-x86_64-pc-windows-msvc.zip",
                    "browser_download_url": "https://github.com/example/repo/releases/download/v1.0.0/binary.zip"
                }
            ]
        }"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(response.to_string()));

        let provider = GithubReleasesProvider {
            http: mock_http,
            platform: Platform {
                os: "windows".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        let result = provider.get_latest().await.unwrap();

        assert_eq!(result.version, "v1.0.0");
        assert!(result.download_url.ends_with(".zip"));
    }
}
