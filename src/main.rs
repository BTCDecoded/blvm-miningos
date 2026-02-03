//! blvm-miningos - MiningOS integration module
//!
//! This module provides bidirectional integration between BLVM and MiningOS,
//! enabling BLVM to register as a MiningOS "rack" (worker) via P2P and
//! query MiningOS via HTTP REST API.

use anyhow::Result;
use blvm_node::module::traits::EventType;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

mod http;
mod data;
mod actions;
mod p2p;
mod manager;
mod config;
mod error;
mod client;
mod nodeapi_ipc;

use manager::MiningOsIntegrationManager;
use client::ModuleClient;
use nodeapi_ipc::NodeApiIpc;

/// Command-line arguments for the module
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Module ID (provided by node)
    #[arg(long)]
    module_id: Option<String>,

    /// IPC socket path (provided by node)
    #[arg(long)]
    socket_path: Option<PathBuf>,

    /// Data directory (provided by node)
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Get module ID (from args or environment)
    let module_id = args.module_id
        .or_else(|| std::env::var("MODULE_NAME").ok())
        .unwrap_or_else(|| "blvm-miningos".to_string());

    // Get data directory
    let data_dir = args.data_dir
        .or_else(|| std::env::var("MODULE_DATA_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("data"));

    info!("blvm-miningos module starting... (module_id: {}, data_dir: {:?})", module_id, data_dir);

    // Get socket path (from args, env, or default)
    let socket_path = args.socket_path
        .or_else(|| std::env::var("BLLVM_MODULE_SOCKET").ok().map(PathBuf::from))
        .or_else(|| std::env::var("MODULE_SOCKET_DIR").ok().map(|d| PathBuf::from(d).join("modules.sock")))
        .unwrap_or_else(|| PathBuf::from("data/modules/modules.sock"));

    // Connect to node
    let mut client = match ModuleClient::connect(
        socket_path,
        module_id.clone(),
        "blvm-miningos".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    ).await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to node: {}", e);
            return Err(anyhow::anyhow!("Connection failed: {}", e));
        }
    };

    // Subscribe to events
    let event_types = vec![
        EventType::BlockMined,
        EventType::BlockTemplateUpdated,
        EventType::MiningDifficultyChanged,
    ];

    if let Err(e) = client.subscribe_events(event_types).await {
        error!("Failed to subscribe to events: {}", e);
        return Err(anyhow::anyhow!("Subscription failed: {}", e));
    }

    // Create NodeAPI wrapper
    let ipc_client = client.get_ipc_client();
    let node_api = Arc::new(NodeApiIpc::new(ipc_client));

    // Load configuration (try multiple possible locations)
    let config_paths = vec![
        data_dir.join("config/miningos.toml"),
        data_dir.join("miningos.toml"),
        std::path::PathBuf::from("./config/miningos.toml"),
        std::path::PathBuf::from("./miningos.toml"),
    ];
    
    let config = config_paths.iter()
        .find(|p| p.exists())
        .and_then(|path| {
            match crate::config::MiningOsConfig::load(path) {
                Ok(cfg) => {
                    info!("Loaded configuration from {:?}", path);
                    Some(cfg)
                }
                Err(e) => {
                    warn!("Failed to load config from {:?}: {}", path, e);
                    None
                }
            }
        })
        .unwrap_or_else(|| {
            info!("No config file found, using defaults");
            crate::config::MiningOsConfig::default()
        });

    // Create integration manager
    let mut manager = MiningOsIntegrationManager::new(config, node_api);

    // Initialize
    if let Err(e) = manager.initialize().await {
        error!("Failed to initialize: {}", e);
        return Err(anyhow::anyhow!("Initialization failed: {}", e));
    }

    // Start
    if let Err(e) = manager.start().await {
        error!("Failed to start: {}", e);
        return Err(anyhow::anyhow!("Start failed: {}", e));
    }

    info!("blvm-miningos module started successfully");

    // Start event loop
    let mut event_handle = tokio::spawn(async move {
        loop {
            // Handle events from node
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Start periodic statistics collection
    let stats_collector = Arc::clone(&manager.stats_collector);
    let stats_handle = if let Some(stats_config) = &manager.config.stats {
        if stats_config.enabled {
            let interval = stats_config.collection_interval_seconds;
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
                loop {
                    interval.tick().await;
                    if let Err(e) = stats_collector.collect().await {
                        tracing::error!("Failed to collect statistics: {}", e);
                    }
                }
            }))
        } else {
            None
        }
    } else {
        None
    };

    // Start periodic template updates
    let template_provider = Arc::clone(&manager.template_provider);
    let template_handle = if let Some(template_config) = &manager.config.template {
        if template_config.enabled {
            let interval = template_config.update_interval_seconds;
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
                loop {
                    interval.tick().await;
                    if let Err(e) = template_provider.update_template().await {
                        tracing::error!("Failed to update template: {}", e);
                    }
                }
            }))
        } else {
            None
        }
    } else {
        None
    };

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    // Cancel background tasks
    event_handle.abort();
    if let Some(handle) = stats_handle {
        handle.abort();
    }
    if let Some(handle) = template_handle {
        handle.abort();
    }

    // Stop
    if let Err(e) = manager.stop().await {
        error!("Failed to stop: {}", e);
    }

    Ok(())
}

