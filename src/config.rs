//! Configuration management for MiningOS integration

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningOsConfig {
    pub miningos: MiningOsSettings,
    pub p2p: Option<P2PConfig>,
    pub http: Option<HttpConfig>,
    pub stats: Option<StatsConfig>,
    pub template: Option<TemplateConfig>,
    pub actions: Option<ActionsConfig>,
    pub things: Option<ThingsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningOsSettings {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub enabled: bool,
    pub orchestrator_rpc_public_key: String,
    pub rack_id: String,
    pub rack_type: String,
    #[serde(default = "default_true")]
    pub auto_register: bool,
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub enabled: bool,
    pub app_node_url: String,
    pub oauth_provider: String,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_callback_url: String,
    #[serde(default = "default_token_cache")]
    pub token_cache_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    pub enabled: bool,
    #[serde(default = "default_collection_interval")]
    pub collection_interval_seconds: u64,
    #[serde(default = "default_hashrate_unit")]
    pub hashrate_unit: String,
    #[serde(default = "default_temperature_unit")]
    pub temperature_unit: String,
    #[serde(default = "default_power_unit")]
    pub power_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub enabled: bool,
    #[serde(default = "default_update_interval")]
    pub update_interval_seconds: u64,
    #[serde(default = "default_true")]
    pub expose_via_http: bool,
    #[serde(default = "default_true")]
    pub expose_via_p2p: bool,
    #[serde(default = "default_cache_duration")]
    pub cache_duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionsConfig {
    pub enabled: bool,
    pub supported_actions: Vec<String>,
    #[serde(default = "default_true")]
    pub require_approval: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingsConfig {
    #[serde(default = "default_true")]
    pub auto_register_miners: bool,
    #[serde(default = "default_miner_tag")]
    pub miner_tag: String,
    #[serde(default = "default_update_interval")]
    pub update_interval_seconds: u64,
}

impl MiningOsConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: MiningOsConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default() -> Self {
        Self {
            miningos: MiningOsSettings { enabled: true },
            p2p: Some(P2PConfig {
                enabled: true,
                orchestrator_rpc_public_key: String::new(),
                rack_id: "blvm-node-001".to_string(),
                rack_type: "miner".to_string(),
                auto_register: true,
                reconnect_interval_seconds: 30,
            }),
            http: Some(HttpConfig {
                enabled: true,
                app_node_url: "http://localhost:3000".to_string(),
                oauth_provider: "google".to_string(),
                oauth_client_id: String::new(),
                oauth_client_secret: String::new(),
                oauth_callback_url: "http://localhost:3000/oauth/google/callback".to_string(),
                token_cache_file: "miningos-token.json".to_string(),
            }),
            stats: Some(StatsConfig {
                enabled: true,
                collection_interval_seconds: 60,
                hashrate_unit: "TH/s".to_string(),
                temperature_unit: "celsius".to_string(),
                power_unit: "watts".to_string(),
            }),
            template: Some(TemplateConfig {
                enabled: true,
                update_interval_seconds: 30,
                expose_via_http: true,
                expose_via_p2p: true,
                cache_duration_seconds: 10,
            }),
            actions: Some(ActionsConfig {
                enabled: true,
                supported_actions: vec![
                    "reboot".to_string(),
                    "setPowerMode".to_string(),
                    "updatePoolConfig".to_string(),
                    "setHashrate".to_string(),
                ],
                require_approval: true,
                timeout_seconds: 120,
            }),
            things: Some(ThingsConfig {
                auto_register_miners: true,
                miner_tag: "t-miner".to_string(),
                update_interval_seconds: 60,
            }),
        }
    }
}

// Default value helpers
fn default_true() -> bool {
    true
}

fn default_reconnect_interval() -> u64 {
    30
}

fn default_token_cache() -> String {
    "miningos-token.json".to_string()
}

fn default_collection_interval() -> u64 {
    60
}

fn default_hashrate_unit() -> String {
    "TH/s".to_string()
}

fn default_temperature_unit() -> String {
    "celsius".to_string()
}

fn default_power_unit() -> String {
    "watts".to_string()
}

fn default_update_interval() -> u64 {
    30
}

fn default_cache_duration() -> u64 {
    10
}

fn default_timeout() -> u64 {
    120
}

fn default_miner_tag() -> String {
    "t-miner".to_string()
}


