//! IPC server for Node.js bridge communication

use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::{Value, json};
use crate::error::{Result, MiningOsError};
use crate::data::{ThingConverter, BlockTemplateProvider, StatisticsCollector};
use crate::actions::ActionHandler;
use tracing::{debug, error, info, warn};

/// IPC server for communicating with Node.js bridge
pub struct BridgeIpcServer {
    socket_path: PathBuf,
    thing_converter: Arc<ThingConverter>,
    template_provider: Arc<BlockTemplateProvider>,
    action_handler: Arc<ActionHandler>,
    stats_collector: Option<Arc<crate::data::StatisticsCollector>>,
}

impl BridgeIpcServer {
    pub fn new(
        socket_path: PathBuf,
        thing_converter: Arc<ThingConverter>,
        template_provider: Arc<BlockTemplateProvider>,
        action_handler: Arc<ActionHandler>,
        stats_collector: Option<Arc<crate::data::StatisticsCollector>>,
    ) -> Self {
        Self {
            socket_path,
            thing_converter,
            template_provider,
            action_handler,
            stats_collector,
        }
    }

    pub async fn listen(&self) -> Result<()> {
        // Remove existing socket if present
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .map_err(|e| MiningOsError::IpcError(format!("Failed to remove old socket: {}", e)))?;
        }

        // Create parent directory if needed
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MiningOsError::IpcError(format!("Failed to create socket directory: {}", e)))?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| MiningOsError::IpcError(format!("Failed to bind socket: {}", e)))?;

        info!("Bridge IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let thing_converter = Arc::clone(&self.thing_converter);
                    let template_provider = Arc::clone(&self.template_provider);
                    let action_handler = Arc::clone(&self.action_handler);
                    let stats_collector = self.stats_collector.as_ref().map(Arc::clone);
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, thing_converter, template_provider, action_handler, stats_collector).await {
                            error!("Error handling bridge connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        mut stream: UnixStream,
        thing_converter: Arc<ThingConverter>,
        template_provider: Arc<BlockTemplateProvider>,
        action_handler: Arc<ActionHandler>,
        stats_collector: Option<Arc<crate::data::StatisticsCollector>>,
    ) -> Result<()> {
        let mut buffer = vec![0u8; 4096];

        loop {
            let n = stream.read(&mut buffer).await
                .map_err(|e| MiningOsError::IpcError(format!("Failed to read: {}", e)))?;

            if n == 0 {
                break; // Connection closed
            }

            // Parse JSON-RPC request
            let request_str = String::from_utf8_lossy(&buffer[..n]);
            let request: Value = serde_json::from_str(&request_str)
                .map_err(|e| MiningOsError::SerializationError(format!("Invalid JSON: {}", e)))?;

            let id = request.get("id").and_then(|v| v.as_u64());
            let method = request.get("method")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MiningOsError::IpcError("Missing method".to_string()))?;
            let params = request.get("params").cloned().unwrap_or(json!({}));

            debug!("Bridge RPC call: {} with params: {:?}", method, params);

            // Handle request
            let result = match method {
                "listThings" => {
                    debug!("IPC: listThings");
                    let things = thing_converter.collect_mining_stats().await?;
                    json!(things)
                }
                "getBlockTemplate" => {
                    debug!("IPC: getBlockTemplate");
                    let template = template_provider.get_template_json().await?;
                    template
                }
                "executeAction" => {
                    debug!("IPC: executeAction");
                    let action_type = params.get("action")
                        .or_else(|| params.get("action_type"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| MiningOsError::IpcError("Missing action type".to_string()))?;
                    let action_params = params.get("params")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    
                    let result = action_handler.execute(action_type, &action_params).await?;
                    json!({
                        "success": result.success,
                        "message": result.message,
                        "data": result.data
                    })
                }
                "ping" => {
                    // Health check
                    json!({
                        "status": "ok",
                        "timestamp": chrono::Utc::now().timestamp(),
                        "module": "blvm-miningos"
                    })
                }
                "getStats" => {
                    // Get current statistics
                    debug!("IPC: getStats");
                    if let Some(ref collector) = stats_collector {
                        match collector.get_stats_json().await {
                            Ok(stats) => stats,
                            Err(e) => {
                                error!("Failed to get stats: {}", e);
                                json!({
                                    "status": "error",
                                    "error": e.to_string(),
                                    "timestamp": chrono::Utc::now().timestamp()
                                })
                            }
                        }
                    } else {
                        json!({
                            "status": "ok",
                            "timestamp": chrono::Utc::now().timestamp(),
                            "module": "blvm-miningos",
                            "note": "Stats collector not available"
                        })
                    }
                }
                _ => {
                    return Err(MiningOsError::IpcError(format!("Unknown method: {}", method)));
                }
            };

            // Send response
            let response = json!({
                "id": id,
                "result": result,
                "error": null
            });

            let response_bytes = serde_json::to_vec(&response)
                .map_err(|e| MiningOsError::SerializationError(e.to_string()))?;
            stream.write_all(&response_bytes).await
                .map_err(|e| MiningOsError::IpcError(format!("Failed to write: {}", e)))?;
            stream.write_all(b"\n").await
                .map_err(|e| MiningOsError::IpcError(format!("Failed to write newline: {}", e)))?;
        }

        Ok(())
    }
}
