//! GitHub schema fetching and caching.
//!
//! This module provides functionality to fetch OCSF schema from the
//! ocsf/ocsf-schema GitHub repository with local caching support.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Default GitHub repository for OCSF schema.
pub const OCSF_GITHUB_REPO: &str = "ocsf/ocsf-schema";

/// Default branch for OCSF schema.
pub const OCSF_DEFAULT_BRANCH: &str = "main";

/// GitHub API base URL.
const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub raw content base URL.
const GITHUB_RAW_BASE: &str = "https://raw.githubusercontent.com";

/// Configuration for GitHub schema fetching.
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    /// Repository in format "owner/repo".
    pub repo: String,
    /// Version tag or branch name.
    pub version: Option<String>,
    /// Local cache directory.
    pub cache_dir: PathBuf,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            repo: OCSF_GITHUB_REPO.to_string(),
            version: None,
            cache_dir: default_cache_dir(),
        }
    }
}

impl GitHubConfig {
    /// Creates a new GitHub config with the specified version.
    pub fn with_version(version: impl Into<String>) -> Self {
        Self {
            version: Some(version.into()),
            ..Default::default()
        }
    }

    /// Sets the cache directory.
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = path.into();
        self
    }

    /// Returns the effective version (tag/branch) to use.
    pub fn effective_version(&self) -> &str {
        self.version.as_deref().unwrap_or(OCSF_DEFAULT_BRANCH)
    }

    /// Returns the cache path for this version.
    pub fn cache_path(&self) -> PathBuf {
        let version = self.effective_version().replace('/', "_");
        self.cache_dir.join(self.repo.replace('/', "_")).join(&version)
    }
}

/// Returns the default cache directory for OCSF schemas.
fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("ocsf-semantic-layer")
        .join("schemas")
}

/// GitHub schema fetcher with caching support.
pub struct GitHubFetcher {
    config: GitHubConfig,
    client: reqwest::Client,
}

impl GitHubFetcher {
    /// Creates a new GitHub fetcher with the given configuration.
    pub fn new(config: GitHubConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder()
                .user_agent("ocsf-semantic-layer")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Creates a new GitHub fetcher with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GitHubConfig::default())
    }

    /// Creates a new GitHub fetcher for a specific version.
    pub fn for_version(version: impl Into<String>) -> Self {
        Self::new(GitHubConfig::with_version(version))
    }

    /// Returns the cache path for the current configuration.
    pub fn cache_path(&self) -> PathBuf {
        self.config.cache_path()
    }

    /// Checks if the schema is cached locally.
    pub fn is_cached(&self) -> bool {
        let cache_path = self.cache_path();
        cache_path.exists() && cache_path.join("version.json").exists()
    }

    /// Fetches the schema from GitHub, using cache if available.
    ///
    /// Returns the path to the local schema directory.
    pub async fn fetch(&self) -> Result<PathBuf> {
        let cache_path = self.cache_path();

        // Check if already cached
        if self.is_cached() {
            return Ok(cache_path);
        }

        // Create cache directory
        std::fs::create_dir_all(&cache_path)
            .with_context(|| format!("Failed to create cache directory: {}", cache_path.display()))?;

        // Fetch schema files
        self.fetch_schema_files(&cache_path).await?;

        Ok(cache_path)
    }

    /// Forces a fresh fetch from GitHub, ignoring cache.
    pub async fn fetch_fresh(&self) -> Result<PathBuf> {
        let cache_path = self.cache_path();

        // Remove existing cache
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path)
                .with_context(|| format!("Failed to remove cache: {}", cache_path.display()))?;
        }

        self.fetch().await
    }

    /// Fetches all schema files from GitHub.
    async fn fetch_schema_files(&self, dest: &Path) -> Result<()> {
        let version = self.config.effective_version();

        // Fetch the main schema files
        self.fetch_file("version.json", dest, version).await?;
        self.fetch_file("dictionary.json", dest, version).await?;
        self.fetch_file("categories.json", dest, version).await?;

        // Fetch directory contents
        self.fetch_directory("objects", dest, version).await?;
        self.fetch_directory("events", dest, version).await?;

        Ok(())
    }

    /// Fetches a single file from GitHub.
    async fn fetch_file(&self, file_path: &str, dest: &Path, version: &str) -> Result<()> {
        let url = format!(
            "{}/{}/{}/{}",
            GITHUB_RAW_BASE, self.config.repo, version, file_path
        );

        let response = self.client.get(&url).send().await
            .with_context(|| format!("Failed to fetch {}", url))?;

        if !response.status().is_success() {
            // File might not exist, which is okay for optional files
            return Ok(());
        }

        let content = response.text().await
            .with_context(|| format!("Failed to read response from {}", url))?;

        let dest_path = dest.join(file_path);
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&dest_path, content)
            .with_context(|| format!("Failed to write {}", dest_path.display()))?;

        Ok(())
    }

    /// Fetches a directory recursively from GitHub.
    async fn fetch_directory(&self, dir_path: &str, dest: &Path, version: &str) -> Result<()> {
        let url = format!(
            "{}/repos/{}/contents/{}?ref={}",
            GITHUB_API_BASE, self.config.repo, dir_path, version
        );

        let response = self.client.get(&url).send().await
            .with_context(|| format!("Failed to fetch directory listing: {}", url))?;

        if !response.status().is_success() {
            // Directory might not exist
            return Ok(());
        }

        let entries: Vec<GitHubContent> = response.json().await
            .with_context(|| format!("Failed to parse directory listing from {}", url))?;

        // Create local directory
        let local_dir = dest.join(dir_path);
        std::fs::create_dir_all(&local_dir)?;

        for entry in entries {
            match entry.content_type.as_str() {
                "file" => {
                    if entry.name.ends_with(".json") {
                        self.fetch_file(&entry.path, dest, version).await?;
                    }
                }
                "dir" => {
                    // Recursively fetch subdirectory
                    Box::pin(self.fetch_directory(&entry.path, dest, version)).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Lists available versions (tags) from the GitHub repository.
    pub async fn list_versions(&self) -> Result<Vec<String>> {
        let url = format!("{}/repos/{}/tags", GITHUB_API_BASE, self.config.repo);

        let response = self.client.get(&url).send().await
            .with_context(|| "Failed to fetch tags from GitHub")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch tags: {}", response.status());
        }

        let tags: Vec<GitHubTag> = response.json().await
            .with_context(|| "Failed to parse tags response")?;

        Ok(tags.into_iter().map(|t| t.name).collect())
    }

    /// Clears the local cache for this configuration.
    pub fn clear_cache(&self) -> Result<()> {
        let cache_path = self.cache_path();
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path)
                .with_context(|| format!("Failed to clear cache: {}", cache_path.display()))?;
        }
        Ok(())
    }
}

/// GitHub API content entry.
#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
}

/// GitHub API tag entry.
#[derive(Debug, Deserialize)]
struct GitHubTag {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_default() {
        let config = GitHubConfig::default();
        assert_eq!(config.repo, OCSF_GITHUB_REPO);
        assert!(config.version.is_none());
        assert_eq!(config.effective_version(), OCSF_DEFAULT_BRANCH);
    }

    #[test]
    fn test_github_config_with_version() {
        let config = GitHubConfig::with_version("v1.4.0");
        assert_eq!(config.effective_version(), "v1.4.0");
    }

    #[test]
    fn test_cache_path() {
        let config = GitHubConfig::with_version("v1.4.0")
            .cache_dir("/tmp/test-cache");
        let cache_path = config.cache_path();
        assert!(cache_path.to_string_lossy().contains("ocsf_ocsf-schema"));
        assert!(cache_path.to_string_lossy().contains("v1.4.0"));
    }

    #[test]
    fn test_fetcher_creation() {
        let fetcher = GitHubFetcher::with_defaults();
        assert!(!fetcher.is_cached());
    }

    #[test]
    fn test_fetcher_for_version() {
        let fetcher = GitHubFetcher::for_version("v1.3.0");
        assert_eq!(fetcher.config.effective_version(), "v1.3.0");
    }
}
