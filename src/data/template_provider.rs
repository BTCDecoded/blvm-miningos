//! Block template provider for MiningOS

use std::sync::Arc;
use std::sync::RwLock;
use crate::error::Result;
use blvm_node::module::traits::NodeAPI;
use blvm_protocol::mining::BlockTemplate;
use tracing::{debug, warn};

/// Block template provider
pub struct BlockTemplateProvider {
    node_api: Arc<dyn NodeAPI>,
    current_template: Arc<RwLock<Option<BlockTemplate>>>,
}

impl BlockTemplateProvider {
    pub fn new(node_api: Arc<dyn NodeAPI>) -> Self {
        Self {
            node_api,
            current_template: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current block template
    pub async fn get_template(&self) -> Result<BlockTemplate> {
        // Try to get cached template first
        {
            let template = self.current_template.read().unwrap();
            if let Some(ref t) = *template {
                return Ok(t.clone());
            }
        }

        // Fetch new template
        self.update_template().await?;

        // Return the newly fetched template
        let template = self.current_template.read().unwrap();
        template.clone().ok_or_else(|| {
            crate::error::MiningOsError::RpcError("Failed to get block template".to_string())
        })
    }

    /// Update template from BLVM
    pub async fn update_template(&self) -> Result<()> {
        debug!("Updating block template from BLVM");

        // Request block template with default rules
        let template = self.node_api.get_block_template(
            vec![], // Default rules
            None,   // No custom coinbase script
            None,   // No custom coinbase address
        ).await
        .map_err(|e| crate::error::MiningOsError::RpcError(format!("Failed to get block template: {}", e)))?;

        // Cache the template
        {
            let mut cached = self.current_template.write().unwrap();
            *cached = Some(template);
        }

        Ok(())
    }

    /// Get template as JSON for HTTP API
    pub async fn get_template_json(&self) -> Result<serde_json::Value> {
        let template = self.get_template().await?;
        
        // Convert BlockTemplate to JSON format similar to Bitcoin Core's getblocktemplate
        let coinbase_value = template.coinbase_tx.outputs.first()
            .map(|out| out.value)
            .unwrap_or(0);
        
        // Convert Hash to hex string (little-endian, reversed for display)
        let prev_hash_hex: String = template.header.prev_block_hash.iter()
            .rev()
            .map(|b| format!("{:02x}", b))
            .collect();
        
        // Convert target (u128) to hex
        let target_hex = format!("{:032x}", template.target);
        
        Ok(serde_json::json!({
            "version": template.header.version,
            "previousblockhash": prev_hash_hex,
            "transactions": template.transactions.len() + 1, // +1 for coinbase
            "coinbasevalue": coinbase_value,
            "target": target_hex,
            "mintime": template.timestamp,
            "curtime": template.timestamp,
            "bits": template.header.bits,
            "height": template.height,
        }))
    }
}

