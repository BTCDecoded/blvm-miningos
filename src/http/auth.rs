//! OAuth2 authentication for MiningOS

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use reqwest::Client;
use serde_json::json;
use crate::error::{MiningOsError, Result};

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
    pub token_cache_file: String,
    pub token_endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCache {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    #[serde(default = "default_token_type")]
    pub token_type: String,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

impl OAuthConfig {
    pub fn new(
        provider: String,
        client_id: String,
        client_secret: String,
        callback_url: String,
        token_cache_file: String,
        token_endpoint: String,
    ) -> Self {
        Self {
            provider,
            client_id,
            client_secret,
            callback_url,
            token_cache_file,
            token_endpoint,
        }
    }

    pub fn load_token<P: AsRef<Path>>(&self, cache_dir: P) -> Result<Option<TokenCache>> {
        let cache_path = cache_dir.as_ref().join(&self.token_cache_file);
        if !cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path)
            .map_err(|e| MiningOsError::ConfigError(format!("Failed to read token cache: {}", e)))?;
        
        let token: TokenCache = serde_json::from_str(&content)
            .map_err(|e| MiningOsError::ConfigError(format!("Failed to parse token cache: {}", e)))?;

        // Check if token is expired
        if let Some(expires_at) = token.expires_at {
            if chrono::Utc::now().timestamp() as u64 >= expires_at {
                return Ok(None); // Token expired
            }
        }

        Ok(Some(token))
    }

    pub fn save_token<P: AsRef<Path>>(&self, cache_dir: P, token: &TokenCache) -> Result<()> {
        let cache_path = cache_dir.as_ref().join(&self.token_cache_file);
        let content = serde_json::to_string_pretty(token)
            .map_err(|e| MiningOsError::SerializationError(e.to_string()))?;
        
        fs::write(&cache_path, content)
            .map_err(|e| MiningOsError::ConfigError(format!("Failed to write token cache: {}", e)))?;

        Ok(())
    }
    
    /// Refresh access token using refresh token
    pub async fn refresh_token<P: AsRef<Path>>(&self, cache_dir: P) -> Result<TokenCache> {
        use reqwest::Client;
        use serde_json::json;
        
        // Load existing token to get refresh token
        let old_token = self.load_token(&cache_dir)?
            .ok_or_else(|| MiningOsError::AuthError("No token to refresh".to_string()))?;
        
        let refresh_token = old_token.refresh_token
            .ok_or_else(|| MiningOsError::AuthError("No refresh token available".to_string()))?;
        
        // Use configured token endpoint, with fallback to provider-specific defaults
        let token_endpoint = if !self.token_endpoint.is_empty() {
            self.token_endpoint.clone()
        } else {
            match self.provider.as_str() {
                "google" => "https://oauth2.googleapis.com/token".to_string(),
                "miningos" | "tether" => {
                    std::env::var("MININGOS_OAUTH_TOKEN_URL")
                        .unwrap_or_else(|_| "https://api.mos.tether.io/oauth/token".to_string())
                }
                custom => {
                    std::env::var(format!("{}_OAUTH_TOKEN_URL", custom.to_uppercase()))
                        .map_err(|_| MiningOsError::AuthError(
                            format!("Unknown OAuth provider: {} (set {}_OAUTH_TOKEN_URL env var or configure token_endpoint)",
                                custom, custom.to_uppercase())
                        ))?
                }
            }
        };
        
        // Build refresh token request
        let client = Client::new();
        let params = json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "refresh_token": refresh_token,
            "grant_type": "refresh_token"
        });
        
        let response = client
            .post(token_endpoint)
            .json(&params)
            .send()
            .await
            .map_err(|e| MiningOsError::HttpError(format!("Token refresh request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MiningOsError::AuthError(
                format!("Token refresh failed with status {}: {}", status, error_text)
            ));
        }
        
        let token_response: serde_json::Value = response.json().await
            .map_err(|e| MiningOsError::HttpError(format!("Failed to parse token response: {}", e)))?;
        
        // Extract new tokens
        let access_token = token_response.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MiningOsError::AuthError("No access_token in refresh response".to_string()))?
            .to_string();
        
        // Refresh token may or may not be included in response
        let new_refresh_token = token_response.get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(Some(refresh_token)); // Reuse old refresh token if not provided
        
        // Calculate expiration time
        let expires_in = token_response.get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(3600); // Default to 1 hour
        
        let expires_at = chrono::Utc::now().timestamp() as u64 + expires_in;
        
        let new_token = TokenCache {
            access_token,
            refresh_token: new_refresh_token,
            expires_at: Some(expires_at),
            token_type: token_response.get("token_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Bearer".to_string()),
        };
        
        // Save new token
        self.save_token(&cache_dir, &new_token)?;
        
        Ok(new_token)
    }
}

