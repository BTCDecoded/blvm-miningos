//! HTTP REST API client for MiningOS

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use crate::error::{MiningOsError, Result};
use crate::http::auth::OAuthConfig;
use crate::http::endpoints::*;

/// HTTP client for MiningOS app-node REST API
pub struct MiningOsHttpClient {
    base_url: String,
    client: Client,
    oauth_config: Arc<OAuthConfig>,
    token: Arc<RwLock<Option<String>>>,
    cache_dir: std::path::PathBuf,
}

impl MiningOsHttpClient {
    pub fn new(
        base_url: String,
        oauth_config: OAuthConfig,
        cache_dir: std::path::PathBuf,
    ) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
            oauth_config: Arc::new(oauth_config),
            token: Arc::new(RwLock::new(None)),
            cache_dir,
        }
    }

    /// Authenticate with OAuth2
    pub async fn authenticate(&self) -> Result<()> {
        // Try to load cached token
        if let Some(token_cache) = self.oauth_config.load_token(&self.cache_dir)? {
            *self.token.write().await = Some(token_cache.access_token.clone());
            info!("Loaded cached OAuth token");
            return Ok(());
        }

        // Try to refresh token if we have a refresh token
        match self.oauth_config.as_ref().refresh_token(&self.cache_dir).await {
            Ok(new_token) => {
                *self.token.write().await = Some(new_token.access_token.clone());
                self.oauth_config.save_token(&self.cache_dir, &new_token)?;
                info!("Refreshed OAuth token");
                return Ok(());
            }
            Err(_) => {
                // Refresh failed, need manual authentication
            }
        }

        // TODO: Implement full OAuth2 authorization code flow
        // For now, check if token is provided via environment variable
        if let Ok(token) = std::env::var("MININGOS_ACCESS_TOKEN") {
            *self.token.write().await = Some(token);
            info!("Loaded OAuth token from environment variable");
            return Ok(());
        }

        warn!("OAuth2 authentication required - set MININGOS_ACCESS_TOKEN environment variable or implement OAuth2 flow");
        Err(MiningOsError::AuthError(
            "OAuth2 authentication required. Set MININGOS_ACCESS_TOKEN environment variable or implement OAuth2 flow".to_string()
        ))
    }

    /// Set authentication token manually (for testing)
    pub async fn set_token(&self, token: String) {
        *self.token.write().await = Some(token);
    }

    /// Get authentication header
    async fn auth_header(&self) -> Result<String> {
        let token = self.token.read().await;
        match token.as_ref() {
            Some(t) => Ok(format!("Bearer {}", t)),
            None => Err(MiningOsError::AuthError("Not authenticated".to_string())),
        }
    }

    /// Make authenticated request
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let auth_header = self.auth_header().await?;

        let mut request = self.client
            .request(method, &url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("HTTP error {}: {}", status, error_text);
            return Err(MiningOsError::HttpError(format!("HTTP {}: {}", status, error_text)));
        }

        let result: T = response.json().await
            .map_err(|e| MiningOsError::HttpError(format!("JSON parse failed: {}", e)))?;
        Ok(result)
    }

    /// List things (miners) from MiningOS
    pub async fn list_things(&self, query: &ThingQuery) -> Result<Vec<Thing>> {
        debug!("Listing things with query: {:?}", query);
        self.request(reqwest::Method::GET, "/auth/list-things", None).await
    }

    /// Get time-series log data
    pub async fn tail_log(&self, params: &TailLogParams) -> Result<Vec<LogEntry>> {
        debug!("Tailing log with params: {:?}", params);
        // TODO: Implement proper query parameter encoding
        self.request(reqwest::Method::GET, "/auth/tail-log", None).await
    }

    /// Submit action
    pub async fn submit_action(&self, action: &Action) -> Result<ActionId> {
        debug!("Submitting action: {:?}", action);
        let body = serde_json::to_value(action)
            .map_err(|e| MiningOsError::SerializationError(e.to_string()))?;
        self.request(reqwest::Method::POST, "/auth/actions/voting", Some(&body)).await
    }

    /// Vote on action
    pub async fn vote_action(&self, id: &str, approve: bool) -> Result<()> {
        debug!("Voting on action {}: {}", id, approve);
        let body = serde_json::json!({
            "id": id,
            "approve": approve
        });
        self.request::<serde_json::Value>(reqwest::Method::PUT, &format!("/auth/actions/voting/{}/vote", id), Some(&body)).await?;
        Ok(())
    }

    /// Get global configuration
    pub async fn get_global_config(&self, fields: Option<&[String]>) -> Result<serde_json::Value> {
        debug!("Getting global config");
        self.request(reqwest::Method::GET, "/auth/global-config", None).await
    }
}

