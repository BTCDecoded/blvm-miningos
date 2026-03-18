//! MiningOS module: unified CLI via #[module] macro.

use blvm_sdk::module::prelude::*;
use blvm_sdk_macros::module;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::manager::MiningOsIntegrationManager;

/// MiningOS module: manager + CLI in one struct.
#[derive(Clone)]
pub struct MiningOsModule {
    pub manager: Arc<RwLock<MiningOsIntegrationManager>>,
}

#[module]
impl MiningOsModule {
    /// Show MiningOS module status (HTTP, P2P, OAuth).
    #[command]
    fn status(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let manager = Arc::clone(&self.manager);
        run_async(async move {
            let m = manager.read().await;
            let http_enabled = m.config.http.as_ref().map(|h| h.enabled).unwrap_or(false);
            let p2p_enabled = m.config.p2p.as_ref().map(|p| p.enabled).unwrap_or(false);
            let stats_enabled = m.config.stats.as_ref().map(|s| s.enabled).unwrap_or(false);
            let template_enabled = m.config.template.as_ref().map(|t| t.enabled).unwrap_or(false);
            Ok(format!(
                "MiningOS module\n\
                 HTTP: {}\n\
                 P2P: {}\n\
                 Stats: {}\n\
                 Template: {}",
                http_enabled, p2p_enabled, stats_enabled, template_enabled
            ))
        })
    }

    /// List configured supported actions.
    #[command]
    fn list_actions(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let manager = Arc::clone(&self.manager);
        run_async(async move {
            let m = manager.read().await;
            let actions = m
                .config
                .actions
                .as_ref()
                .map(|a| a.supported_actions.clone())
                .unwrap_or_default();
            if actions.is_empty() {
                Ok("No actions configured.\n\
                    Set [actions] supported_actions in config.toml."
                    .into())
            } else {
                Ok(format!(
                    "Supported actions ({}):\n{}",
                    actions.len(),
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, a)| format!("  {}. {}", i + 1, a))
                        .collect::<Vec<_>>()
                        .join("\n"),
                ))
            }
        })
    }

    /// Show OAuth authentication status.
    #[command]
    fn oauth_status(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let manager = Arc::clone(&self.manager);
        run_async(async move {
            let m = manager.read().await;
            if let Some(client) = m.get_http_client() {
                let (_, status) = client.oauth_status().await;
                Ok(status)
            } else {
                Ok("HTTP/OAuth not enabled. Enable [http] in config.toml.".into())
            }
        })
    }

    /// Trigger an action (e.g. reboot, setPowerMode).
    #[command]
    fn trigger_action(&self, _ctx: &InvocationContext, action_type: String) -> Result<String, ModuleError> {
        let action_type = action_type.trim();
        if action_type.is_empty() {
            return Ok("Usage: blvm miningos trigger-action <action_type> (e.g. reboot, setPowerMode)".into());
        }
        let manager = Arc::clone(&self.manager);
        run_async(async move {
            let m = manager.read().await;
            let handler = m.get_action_handler();
            drop(m);
            let r = handler
                .execute(action_type, &serde_json::json!({}))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(format!(
                "Action {}: {}",
                if r.success { "OK" } else { "FAILED" },
                r.message
            ))
        })
    }

    /// List known miners/things.
    #[command]
    fn list_miners(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let manager = Arc::clone(&self.manager);
        run_async(async move {
            let m = manager.read().await;
            let converter = m.get_thing_converter();
            drop(m);
            match converter.collect_mining_stats().await {
                Ok(things) => {
                    if things.is_empty() {
                        Ok("No miners/things registered.".into())
                    } else {
                        Ok(format!(
                            "Miners/things ({}):\n{}",
                            things.len(),
                            things
                                .iter()
                                .enumerate()
                                .map(|(i, t)| format!("  {}. {} ({})", i + 1, t.id, t.thing_type))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ))
                    }
                }
                Err(e) => Ok(format!("Failed to list miners: {}", e)),
            }
        })
    }
}
