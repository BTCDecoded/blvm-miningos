//! Module API for inter-module communication
//!
//! Exposes trigger_action, get_miner_list, get_action_status for dashboards and automation.

use blvm_node::module::ipc::protocol::EventPayload;
use blvm_node::module::inter_module::api::ModuleAPI;
use blvm_node::module::traits::{EventType, ModuleError, NodeAPI};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Last action info for get_action_status
#[derive(Debug, Clone, serde::Serialize)]
struct LastActionInfo {
    action_type: String,
    success: bool,
    message: String,
}

/// MiningOS module API for other modules
pub struct MiningOsModuleApi {
    action_handler: Arc<crate::actions::ActionHandler>,
    thing_converter: Arc<crate::data::ThingConverter>,
    last_action: Arc<RwLock<Option<LastActionInfo>>>,
    node_api: Option<Arc<dyn NodeAPI>>,
}

impl MiningOsModuleApi {
    /// Create a new MiningOS module API
    pub fn new(
        action_handler: Arc<crate::actions::ActionHandler>,
        thing_converter: Arc<crate::data::ThingConverter>,
    ) -> Self {
        Self {
            action_handler,
            thing_converter,
            last_action: Arc::new(RwLock::new(None)),
            node_api: None,
        }
    }

    /// Create with NodeAPI for event publishing
    pub fn with_node_api(
        action_handler: Arc<crate::actions::ActionHandler>,
        thing_converter: Arc<crate::data::ThingConverter>,
        node_api: Arc<dyn NodeAPI>,
    ) -> Self {
        Self {
            action_handler,
            thing_converter,
            last_action: Arc::new(RwLock::new(None)),
            node_api: Some(node_api),
        }
    }
}

#[async_trait::async_trait]
impl ModuleAPI for MiningOsModuleApi {
    async fn handle_request(
        &self,
        method: &str,
        params: &[u8],
        _caller_module_id: &str,
    ) -> Result<Vec<u8>, ModuleError> {
        match method {
            "trigger_action" => {
                let params_json: serde_json::Value = serde_json::from_slice(params)
                    .unwrap_or(serde_json::json!({}));
                let action_type = params_json
                    .get("action_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if action_type.is_empty() {
                    return Err(ModuleError::OperationError(
                        "trigger_action requires action_type parameter".to_string(),
                    ));
                }
                let result = self
                    .action_handler
                    .execute(action_type, &params_json)
                    .await
                    .map_err(|e| ModuleError::OperationError(format!("Action failed: {}", e)))?;
                let info = LastActionInfo {
                    action_type: action_type.to_string(),
                    success: result.success,
                    message: result.message.clone(),
                };
                *self.last_action.write().await = Some(info);

                let target = params_json
                    .get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("all");
                let action_id = format!("{}_{}", action_type, std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis());
                if let Some(ref api) = self.node_api {
                    let payload = EventPayload::ActionExecuted {
                        action_id: action_id.clone(),
                        action_type: action_type.to_string(),
                        target: target.to_string(),
                        success: result.success,
                    };
                    let _ = api.publish_event(EventType::ActionExecuted, payload).await;
                }

                serde_json::to_vec(&serde_json::json!({
                    "success": result.success,
                    "message": result.message,
                    "data": result.data
                }))
                .map_err(|e| ModuleError::OperationError(format!("Serialization error: {}", e)))
            }
            "get_miner_list" => {
                let things = self
                    .thing_converter
                    .collect_mining_stats()
                    .await
                    .map_err(|e| ModuleError::OperationError(format!("Failed to list miners: {}", e)))?;
                let miners: Vec<serde_json::Value> = things
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "type": t.thing_type,
                            "tags": t.tags
                        })
                    })
                    .collect();
                serde_json::to_vec(&serde_json::json!({ "miners": miners }))
                    .map_err(|e| ModuleError::OperationError(format!("Serialization error: {}", e)))
            }
            "get_action_status" => {
                let last = self.last_action.read().await;
                let status = last.as_ref().map(|a| {
                    serde_json::json!({
                        "last_action_type": a.action_type,
                        "success": a.success,
                        "message": a.message,
                        "tracking": "last_action_only"
                    })
                }).unwrap_or(serde_json::json!({
                    "last_action_type": serde_json::Value::Null,
                    "tracking": "last_action_only",
                    "message": "No action has been triggered yet"
                }));
                serde_json::to_vec(&status)
                    .map_err(|e| ModuleError::OperationError(format!("Serialization error: {}", e)))
            }
            _ => Err(ModuleError::OperationError(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }

    fn list_methods(&self) -> Vec<String> {
        vec![
            "trigger_action".to_string(),
            "get_miner_list".to_string(),
            "get_action_status".to_string(),
        ]
    }

    fn api_version(&self) -> u32 {
        1
    }
}
