//! Basic usage example for blvm-miningos module
//!
//! This example demonstrates how to use the MiningOS integration module
//! in a standalone context (for testing).

use blvm_miningos::*;
use std::sync::Arc;
use blvm_node::module::traits::NodeAPI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("blvm_miningos=debug,info")
        .init();

    println!("blvm-miningos basic usage example");
    println!("===================================");

    // Load configuration
    let config = MiningOsConfig::default();
    println!("✓ Loaded default configuration");

    // Note: In a real scenario, you would have a NodeAPI implementation
    // For this example, we'll just show the structure
    println!("\nModule structure:");
    println!("  - HTTP client: {}", if config.http.as_ref().map(|h| h.enabled).unwrap_or(false) { "enabled" } else { "disabled" });
    println!("  - P2P bridge: {}", if config.p2p.as_ref().map(|p| p.enabled).unwrap_or(false) { "enabled" } else { "disabled" });
    println!("  - Statistics: {}", if config.stats.as_ref().map(|s| s.enabled).unwrap_or(false) { "enabled" } else { "disabled" });
    println!("  - Template: {}", if config.template.as_ref().map(|t| t.enabled).unwrap_or(false) { "enabled" } else { "disabled" });

    println!("\nTo use this module:");
    println!("1. Configure MiningOS connection in data/config/miningos.toml");
    println!("2. Set MININGOS_ACCESS_TOKEN environment variable (or implement OAuth2 flow)");
    println!("3. Start the module via BLVM node module system");
    println!("4. The module will automatically:");
    println!("   - Connect to BLVM node via IPC");
    println!("   - Register with MiningOS orchestrator (if P2P enabled)");
    println!("   - Start collecting statistics");
    println!("   - Provide block templates");

    Ok(())
}


