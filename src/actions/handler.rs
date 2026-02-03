//! Action execution handler

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crate::error::Result;

/// Action execution result
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Action executor trait
#[async_trait]
pub trait ActionExecutor: Send + Sync {
    async fn execute(&self, params: &serde_json::Value) -> Result<ActionResult>;
}

/// Action handler
pub struct ActionHandler {
    executors: HashMap<String, Box<dyn ActionExecutor>>,
    node_api: Option<std::sync::Arc<dyn blvm_node::module::traits::NodeAPI>>,
}

impl ActionHandler {
    pub fn new() -> Self {
        Self::with_node_api(None)
    }
    
    pub fn with_node_api(node_api: Option<std::sync::Arc<dyn blvm_node::module::traits::NodeAPI>>) -> Self {
        let mut handler = Self {
            executors: HashMap::new(),
            node_api: node_api.clone(),
        };
        
        // Register default executors
        handler.register_executor("reboot".to_string(), Box::new(crate::actions::executors::RebootExecutor));
        handler.register_executor("setPowerMode".to_string(), Box::new(crate::actions::executors::SetPowerModeExecutor));
        handler.register_executor("updatePoolConfig".to_string(), 
            Box::new(crate::actions::executors::UpdatePoolConfigExecutor::new(node_api.clone())));
        handler.register_executor("setHashrate".to_string(), Box::new(crate::actions::executors::SetHashrateExecutor));
        
        handler
    }

    pub fn register_executor(&mut self, action_type: String, executor: Box<dyn ActionExecutor>) {
        self.executors.insert(action_type, executor);
    }

    pub async fn execute(&self, action_type: &str, params: &serde_json::Value) -> Result<ActionResult> {
        match self.executors.get(action_type) {
            Some(executor) => executor.execute(params).await,
            None => Ok(ActionResult {
                success: false,
                message: format!("Unknown action type: {}", action_type),
                data: None,
            }),
        }
    }
}

