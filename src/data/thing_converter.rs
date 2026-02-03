//! Convert BLVM data to MiningOS format

use std::sync::Arc;
use crate::error::Result;
use crate::http::endpoints::{Thing, ThingSnapshot, ThingStats, MinerStats};
use blvm_node::module::traits::NodeAPI;
use tracing::{debug, warn};

/// Convert BLVM mining data to MiningOS things
pub struct ThingConverter {
    node_api: Arc<dyn NodeAPI>,
    rack_id: String,
}

impl ThingConverter {
    pub fn new(node_api: Arc<dyn NodeAPI>, rack_id: String) -> Self {
        Self { node_api, rack_id }
    }

    /// Convert BLVM node to MiningOS thing
    /// 
    /// Since BLVM is a node-level system, we treat the node itself as a "miner" thing
    pub async fn convert_node_to_thing(&self) -> Result<Thing> {
        // Get chain tip and height for status
        // Note: These calls may fail if node is not fully synced, so we handle errors gracefully
        let _chain_tip = self.node_api.get_chain_tip().await.ok();
        let _height = self.node_api.get_block_height().await.ok();

        // For now, we'll create a thing representing the BLVM node
        // In the future, if BLVM supports multiple miners, we can query them individually
        let thing_id = format!("blvm-node-{}", self.rack_id);

        Ok(Thing {
            id: thing_id,
            thing_type: "miner".to_string(),
            tags: vec!["t-miner".to_string(), "blvm".to_string(), "node".to_string()],
            last: Some(ThingSnapshot {
                snap: ThingStats {
                    stats: MinerStats {
                        status: "online".to_string(), // Node is online if we can query it
                        hashrate: self.get_hashrate_estimate().await, // Try to estimate from chain info
                        temperature: None, // Not available at node level (requires hardware monitoring)
                        power: None, // Not available at node level (requires hardware monitoring)
                    },
                },
            }),
        })
    }

    /// Collect all mining things from BLVM
    /// 
    /// Currently returns a single thing representing the BLVM node
    pub async fn collect_mining_stats(&self) -> Result<Vec<Thing>> {
        debug!("Collecting mining stats from BLVM node");
        
        // Convert node to thing
        let node_thing = self.convert_node_to_thing().await?;
        
        Ok(vec![node_thing])
    }

    /// Get hashrate estimate from chain info
    /// This is a rough estimate based on difficulty and block times
    async fn get_hashrate_estimate(&self) -> Option<u64> {
        // Try to get chain info for difficulty-based estimate
        if let Ok(chain_info) = self.node_api.get_chain_info().await {
            // Rough estimate: difficulty * 2^32 / 600 seconds (10 min block time)
            // This is a very rough approximation
            // Note: difficulty is u32, not Option<u32>
            let difficulty = chain_info.difficulty;
            // Convert to hashrate estimate (H/s)
            // Note: This is just an estimate, not actual measured hashrate
            let estimate = (difficulty as u64) * 4294967296u64 / 600;
            return Some(estimate);
        }
        None
    }

    /// Convert a specific miner ID to thing
    pub async fn convert_miner_to_thing(&self, miner_id: &str) -> Result<Thing> {
        // For now, if miner_id matches our rack_id, return node thing
        if miner_id == self.rack_id || miner_id.starts_with("blvm-node-") {
            return self.convert_node_to_thing().await;
        }

        // Otherwise, create a placeholder thing
        warn!("Unknown miner ID: {}, creating placeholder", miner_id);
        Ok(Thing {
            id: format!("blvm-miner-{}", miner_id),
            thing_type: "miner".to_string(),
            tags: vec!["t-miner".to_string(), "blvm".to_string()],
            last: Some(ThingSnapshot {
                snap: ThingStats {
                    stats: MinerStats {
                        status: "offline".to_string(),
                        hashrate: None,
                        temperature: None,
                        power: None,
                    },
                },
            }),
        })
    }
}

