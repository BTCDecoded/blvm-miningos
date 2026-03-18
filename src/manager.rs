//! Integration manager for MiningOS module

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use crate::error::{MiningOsError, Result};
use crate::config::MiningOsConfig;
use crate::http::MiningOsHttpClient;
use crate::data::{ThingConverter, StatisticsCollector, BlockTemplateProvider};
use crate::actions::ActionHandler;
use crate::p2p::{BridgeIpcServer, BridgeManager};
use blvm_node::module::traits::NodeAPI;

/// Main integration manager
pub struct MiningOsIntegrationManager {
    pub config: MiningOsConfig,
    pub http_client: Option<Arc<MiningOsHttpClient>>,
    pub thing_converter: Arc<ThingConverter>,
    pub stats_collector: Arc<StatisticsCollector>,
    pub template_provider: Arc<BlockTemplateProvider>,
    pub action_handler: Arc<ActionHandler>,
    bridge_manager: Option<Arc<RwLock<BridgeManager>>>,
    bridge_server: Option<Arc<BridgeIpcServer>>,
}

impl MiningOsIntegrationManager {
    pub fn new(config: MiningOsConfig, node_api: Arc<dyn NodeAPI>) -> Self {
        let rack_id = config.p2p.as_ref()
            .map(|p| p.rack_id.clone())
            .unwrap_or_else(|| "blvm-node-001".to_string());
        
        Self {
            config,
            http_client: None,
            thing_converter: Arc::new(ThingConverter::new(Arc::clone(&node_api), rack_id)),
            stats_collector: Arc::new(StatisticsCollector::new(Arc::clone(&node_api))),
            template_provider: Arc::new(BlockTemplateProvider::new(Arc::clone(&node_api))),
            action_handler: Arc::new(ActionHandler::with_node_api(Some(Arc::clone(&node_api)))),
            bridge_manager: None,
            bridge_server: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing MiningOS integration module");

        // Initialize HTTP client if enabled
        if let Some(http_config) = &self.config.http {
            if http_config.enabled {
                let token_endpoint = http_config
                    .oauth_token_url
                    .clone()
                    .unwrap_or_else(|| format!("{}/oauth/token", http_config.app_node_url.trim_end_matches('/')));
                let oauth_config = crate::http::auth::OAuthConfig::new(
                    http_config.oauth_provider.clone(),
                    http_config.oauth_client_id.clone(),
                    http_config.oauth_client_secret.clone(),
                    http_config.oauth_callback_url.clone(),
                    http_config.token_cache_file.clone(),
                    token_endpoint,
                );

                // Use data directory from config or default
                let cache_dir = std::env::var("BLVM_DATA_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("./data"));
                let client = Arc::new(MiningOsHttpClient::new(
                    http_config.app_node_url.clone(),
                    oauth_config,
                    cache_dir,
                ));

                self.http_client = Some(client);
            }
        }

        // Initialize P2P bridge if enabled
        if let Some(p2p_config) = &self.config.p2p {
            if p2p_config.enabled {
                // Use data directory for socket path
                let data_dir = std::env::var("MODULE_DATA_DIR")
                    .or_else(|_| std::env::var("BLVM_DATA_DIR"))
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("./data"));
                let socket_path = data_dir.join("bridge.sock");
                
                // Set environment variable for bridge worker
                std::env::set_var("BLVM_RUST_SOCKET_PATH", socket_path.to_string_lossy().to_string());
                std::env::set_var("BLVM_RACK_ID", p2p_config.rack_id.clone());
                
                let bridge_manager = Arc::new(RwLock::new(BridgeManager::new(socket_path.clone())));
                let bridge_server = Arc::new(BridgeIpcServer::new(
                    socket_path,
                    Arc::clone(&self.thing_converter),
                    Arc::clone(&self.template_provider),
                    Arc::clone(&self.action_handler),
                    Some(Arc::clone(&self.stats_collector)),
                ));

                self.bridge_manager = Some(bridge_manager);
                self.bridge_server = Some(bridge_server);
            }
        }

        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting MiningOS integration module");

        // Authenticate HTTP client
        if let Some(client) = &self.http_client {
            if let Err(e) = client.authenticate().await {
                error!("Failed to authenticate HTTP client: {}", e);
                // Continue anyway - authentication might be manual
            }
        }

        // Start bridge server in background
        if let Some(server) = &self.bridge_server {
            let server_clone = Arc::clone(server);
            tokio::spawn(async move {
                if let Err(e) = server_clone.listen().await {
                    error!("Bridge server error: {}", e);
                }
            });
        }

        // Spawn bridge process
        if let Some(manager) = &self.bridge_manager {
            let mut mgr = manager.write().await;
            if let Err(e) = mgr.spawn_bridge().await {
                error!("Failed to spawn bridge process: {}", e);
                // Continue anyway - bridge is optional
            }
        }

        Ok(())
    }
    
    /// Get HTTP client (if enabled)
    pub fn get_http_client(&self) -> Option<Arc<MiningOsHttpClient>> {
        self.http_client.clone()
    }
    
    /// Get thing converter
    pub fn get_thing_converter(&self) -> Arc<ThingConverter> {
        Arc::clone(&self.thing_converter)
    }
    
    /// Get template provider
    pub fn get_template_provider(&self) -> Arc<BlockTemplateProvider> {
        Arc::clone(&self.template_provider)
    }
    
    /// Get action handler
    pub fn get_action_handler(&self) -> Arc<ActionHandler> {
        Arc::clone(&self.action_handler)
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping MiningOS integration module");

        // Stop bridge process
        if let Some(manager) = &self.bridge_manager {
            let mut mgr = manager.write().await;
            if let Err(e) = mgr.stop_bridge().await {
                error!("Failed to stop bridge process: {}", e);
            }
        }

        Ok(())
    }
}

