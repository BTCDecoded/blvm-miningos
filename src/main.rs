//! blvm-miningos - MiningOS integration module
//!
//! When spawned by the node: reads MODULE_ID, SOCKET_PATH, DATA_DIR from env.
//! For manual testing: blvm-miningos --module-id <id> --socket-path <path> --data-dir <dir>

use anyhow::Result;
use blvm_miningos::MiningOsModule;
use blvm_miningos::config::MiningOsConfig;
use blvm_miningos::{api::MiningOsModuleApi, MiningOsIntegrationManager};
use blvm_node::module::integration::ModuleIntegration;
use blvm_node::module::ipc::protocol::{
    InvocationResultMessage, InvocationResultPayload, InvocationType,
};
use blvm_node::module::traits::EventType;
use blvm_sdk::module::{ModuleBootstrap, ModuleDb};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let bootstrap = ModuleBootstrap::from_env_or_defaults(
        "blvm-miningos",
        "data/modules/blvm-miningos.sock",
        "data/modules/blvm-miningos",
    );

    info!(
        "blvm-miningos module starting... (module_id: {}, data_dir: {:?})",
        bootstrap.module_id, bootstrap.data_dir
    );

    let mut integration = ModuleIntegration::connect(
        bootstrap.socket_path.clone(),
        bootstrap.module_id.clone(),
        "blvm-miningos".into(),
        env!("CARGO_PKG_VERSION").into(),
        Some(MiningOsModule::cli_spec()),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Connection failed: {}", e))?;

    integration
        .subscribe_events(vec![
            EventType::BlockMined,
            EventType::BlockTemplateUpdated,
            EventType::MiningDifficultyChanged,
        ])
        .await
        .map_err(|e| anyhow::anyhow!("Subscription failed: {}", e))?;

    let node_api = integration.node_api();

    let config_paths = [
        bootstrap.data_dir.join("config.toml"),
        bootstrap.data_dir.join("config/miningos.toml"),
        bootstrap.data_dir.join("miningos.toml"),
        std::path::PathBuf::from("./config/miningos.toml"),
        std::path::PathBuf::from("./miningos.toml"),
    ];

    let config = config_paths
        .iter()
        .find(|p| p.exists())
        .map(|path| {
            info!("Loaded configuration from {:?}", path);
            MiningOsConfig::load(path).unwrap_or_else(|e| {
                warn!("Failed to load config from {:?}: {}, using defaults", path, e);
                MiningOsConfig::default()
            })
        })
        .unwrap_or_else(|| {
            info!("No config file found, using defaults");
            MiningOsConfig::default()
        });

    let mut manager = MiningOsIntegrationManager::new(config, node_api.clone());

    if let Err(e) = manager.initialize().await {
        error!("Failed to initialize: {}", e);
        return Err(anyhow::anyhow!("Initialization failed: {}", e));
    }

    if let Err(e) = manager.start().await {
        error!("Failed to start: {}", e);
        return Err(anyhow::anyhow!("Start failed: {}", e));
    }

    let manager = Arc::new(RwLock::new(manager));
    let miningos_api = Arc::new(MiningOsModuleApi::with_node_api(
        manager.read().await.get_action_handler(),
        manager.read().await.get_thing_converter(),
        node_api.clone(),
    ));
    if let Err(e) = node_api.register_module_api(miningos_api).await {
        warn!("Failed to register miningos module API: {}", e);
    }

    info!("blvm-miningos module started successfully");

    let db: Arc<dyn blvm_node::storage::database::Database> = match ModuleDb::open(&bootstrap.data_dir)
        .or_else(|_| ModuleDb::open(std::env::temp_dir().join("blvm-miningos")))
    {
        Ok(module_db) => module_db.as_db(),
        Err(_) => {
            let dir = std::env::temp_dir().join("blvm-miningos").join("db");
            std::fs::create_dir_all(&dir).ok();
            Arc::from(
                blvm_node::storage::database::create_database(
                    &dir,
                    blvm_node::storage::database::DatabaseBackend::Redb,
                    None,
                )
                .expect("fallback temp db"),
            )
        }
    };
    let invocation_ctx = blvm_sdk::module::runner::InvocationContext::new(Arc::clone(&db));
    let module = MiningOsModule {
        manager: Arc::clone(&manager),
    };

    let mut invocation_rx = integration.invocation_receiver().expect("CLI spec provided");

    let mut event_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    let stats_collector = Arc::clone(&manager.read().await.stats_collector);
    let stats_handle = if let Some(stats_config) = &manager.read().await.config.stats {
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

    let template_provider = Arc::clone(&manager.read().await.template_provider);
    let template_handle = if let Some(template_config) = &manager.read().await.config.template {
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

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down...");
                break;
            }
            inv = invocation_rx.recv() => {
                if let Some((invocation, result_tx)) = inv {
                    let result = match &invocation.invocation_type {
                        InvocationType::Cli { subcommand, args } => {
                            match module.dispatch_cli(&invocation_ctx, subcommand, args) {
                                Ok(stdout) => InvocationResultMessage {
                                    correlation_id: invocation.correlation_id,
                                    success: true,
                                    payload: Some(InvocationResultPayload::Cli {
                                        stdout,
                                        stderr: String::new(),
                                        exit_code: 0,
                                    }),
                                    error: None,
                                },
                                Err(e) => InvocationResultMessage {
                                    correlation_id: invocation.correlation_id,
                                    success: false,
                                    payload: None,
                                    error: Some(e.to_string()),
                                },
                            }
                        }
                        _ => InvocationResultMessage {
                            correlation_id: invocation.correlation_id,
                            success: false,
                            payload: None,
                            error: Some("RPC not implemented".to_string()),
                        },
                    };
                    let _ = result_tx.send(result);
                } else {
                    info!("Invocation channel closed, module unloading");
                    break;
                }
            }
        }
    }

    info!("Shutting down...");

    event_handle.abort();
    if let Some(handle) = stats_handle {
        handle.abort();
    }
    if let Some(handle) = template_handle {
        handle.abort();
    }

    if let Err(e) = manager.write().await.stop().await {
        error!("Failed to stop: {}", e);
    }

    Ok(())
}
