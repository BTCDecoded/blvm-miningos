//! MiningOS HTTP API endpoint types

use serde::{Deserialize, Serialize};

/// Query for listing things
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingQuery {
    pub query: Option<serde_json::Value>,
    pub fields: Option<serde_json::Value>,
    pub sort: Option<serde_json::Value>,
    pub limit: Option<u64>,
}

/// MiningOS Thing (device/miner)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thing {
    pub id: String,
    #[serde(rename = "type")]
    pub thing_type: String,
    pub tags: Vec<String>,
    pub last: Option<ThingSnapshot>,
}

/// Thing snapshot with stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingSnapshot {
    pub snap: ThingStats,
}

/// Thing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingStats {
    pub stats: MinerStats,
}

/// Miner statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub status: String,
    pub hashrate: Option<u64>,
    pub temperature: Option<f64>,
    pub power: Option<u64>,
}

/// Tail log parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailLogParams {
    #[serde(rename = "type")]
    pub log_type: String,
    pub key: Option<String>,
    pub tag: Option<String>,
    pub fields: Option<Vec<String>>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub limit: Option<u64>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub ts: u64,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Action to submit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub target: String,
    pub params: serde_json::Value,
}

/// Action ID response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionId {
    pub id: String,
}


