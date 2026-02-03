//! Collect mining statistics from BLVM

use std::sync::Arc;
use crate::error::Result;
use blvm_node::module::traits::NodeAPI;
use tracing::debug;

/// Statistics collector for mining data
pub struct StatisticsCollector {
    node_api: Arc<dyn NodeAPI>,
}

impl StatisticsCollector {
    pub fn new(node_api: Arc<dyn NodeAPI>) -> Self {
        Self { node_api }
    }

    /// Collect mining statistics from BLVM node
    pub async fn collect(&self) -> Result<()> {
        debug!("Collecting mining statistics");

        // Get chain state
        let chain_tip = self.node_api.get_chain_tip().await
            .map_err(|e| crate::error::MiningOsError::RpcError(format!("Failed to get chain tip: {}", e)))?;
        let height = self.node_api.get_block_height().await
            .map_err(|e| crate::error::MiningOsError::RpcError(format!("Failed to get block height: {}", e)))?;

        let chain_tip_hex = chain_tip.iter()
            .rev()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        debug!("Chain tip: {}, Height: {}", chain_tip_hex, height);

        // Try to get chain info for additional statistics
        if let Ok(chain_info) = self.node_api.get_chain_info().await {
            debug!("Chain info: difficulty={}, height={}", 
                chain_info.difficulty, 
                chain_info.height);
        }

        // Try to get network stats
        if let Ok(network_stats) = self.node_api.get_network_stats().await {
            debug!("Network stats: peers={}, hash_rate={:.2} H/s", 
                network_stats.peer_count,
                network_stats.hash_rate);
        }

        // Try to get mempool size
        if let Ok(mempool_size) = self.node_api.get_mempool_size().await {
            debug!("Mempool size: {} transactions, {} bytes", 
                mempool_size.transaction_count,
                mempool_size.size_bytes);
        }

        // Try to query Stratum V2 module for mining statistics if available
        // This would require the module to be loaded and callable
        // For now, we log that this is attempted
        debug!("Note: Mining-specific stats (hashrate, shares) require Stratum V2 module integration");

        debug!("Statistics collection complete (height: {})", height);
        Ok(())
    }
    
    /// Get current mining statistics as JSON
    pub async fn get_stats_json(&self) -> Result<serde_json::Value> {
        let chain_tip = self.node_api.get_chain_tip().await
            .map_err(|e| crate::error::MiningOsError::RpcError(format!("Failed to get chain tip: {}", e)))?;
        let height = self.node_api.get_block_height().await
            .map_err(|e| crate::error::MiningOsError::RpcError(format!("Failed to get block height: {}", e)))?;
        
        let chain_tip_hex = chain_tip.iter()
            .rev()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        
        let mut stats = serde_json::json!({
            "chain_tip": chain_tip_hex,
            "height": height,
            "timestamp": chrono::Utc::now().timestamp(),
        });

        // Add chain info if available
        if let Ok(chain_info) = self.node_api.get_chain_info().await {
            stats["difficulty"] = serde_json::json!(chain_info.difficulty);
            stats["height"] = serde_json::json!(chain_info.height);
            stats["is_synced"] = serde_json::json!(chain_info.is_synced);
        }

        // Add network stats if available
        if let Ok(network_stats) = self.node_api.get_network_stats().await {
            stats["peer_count"] = serde_json::json!(network_stats.peer_count);
            stats["hash_rate"] = serde_json::json!(network_stats.hash_rate);
            stats["bytes_sent"] = serde_json::json!(network_stats.bytes_sent);
            stats["bytes_received"] = serde_json::json!(network_stats.bytes_received);
        }

        // Add mempool size if available
        if let Ok(mempool_size) = self.node_api.get_mempool_size().await {
            stats["mempool_transaction_count"] = serde_json::json!(mempool_size.transaction_count);
            stats["mempool_size_bytes"] = serde_json::json!(mempool_size.size_bytes);
            stats["mempool_total_fee_sats"] = serde_json::json!(mempool_size.total_fee_sats);
        }

        Ok(stats)
    }
}

