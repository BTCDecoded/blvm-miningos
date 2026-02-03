//! Bridge process manager

use std::path::PathBuf;
use tokio::process::{Child, Command};
use crate::error::{Result, MiningOsError};
use tracing::{debug, error, info, warn};

/// Manages Node.js bridge process
pub struct BridgeManager {
    process: Option<Child>,
    socket_path: PathBuf,
    bridge_path: PathBuf,
}

impl BridgeManager {
    pub fn new(socket_path: PathBuf) -> Self {
        // Try to find bridge worker.js
        // Check multiple possible locations
        let mut possible_paths = vec![
            PathBuf::from("bridge/worker.js"),
            PathBuf::from("./bridge/worker.js"),
            PathBuf::from("../bridge/worker.js"),
        ];
        
        // Try to find from current executable location
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                possible_paths.push(parent.join("bridge/worker.js"));
            }
        }
        
        let bridge_path = possible_paths.iter()
            .find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| PathBuf::from("bridge/worker.js"));
        
        Self {
            process: None,
            socket_path,
            bridge_path,
        }
    }

    pub async fn spawn_bridge(&mut self) -> Result<()> {
        if self.process.is_some() {
            debug!("Bridge process already running");
            return Ok(());
        }

        info!("Spawning Node.js bridge process");

        // Check if bridge file exists
        if !self.bridge_path.exists() {
            warn!("Bridge worker not found at {:?}, skipping bridge spawn", self.bridge_path);
            return Ok(()); // Not an error - bridge is optional
        }

        // Spawn Node.js process
               // Use node from PATH, or try common locations
               let node_cmd = std::env::var("NODE_PATH")
                   .map(PathBuf::from)
                   .unwrap_or_else(|_| PathBuf::from("node"));
               
               let mut cmd = Command::new(&node_cmd);
               cmd.arg(&self.bridge_path)
                   .env("BLVM_RUST_SOCKET_PATH", &self.socket_path)
                   .env("NODE_ENV", "production")
                   .current_dir(self.bridge_path.parent().unwrap_or_else(|| std::path::Path::new(".")))
                   .stdout(std::process::Stdio::piped())
                   .stderr(std::process::Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| MiningOsError::P2PError(format!("Failed to spawn bridge: {}", e)))?;

        self.process = Some(child);
        info!("Bridge process spawned successfully");

        Ok(())
    }

    pub async fn stop_bridge(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            info!("Stopping bridge process");
            
            // Try graceful shutdown first
            if let Err(e) = process.kill().await {
                error!("Failed to kill bridge process: {}", e);
            }

            // Wait for process to exit
            let _ = process.wait().await;
            info!("Bridge process stopped");
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.as_ref().map(|p| p.id().is_some()).unwrap_or(false)
    }
}

