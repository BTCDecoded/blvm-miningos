//! blvm-miningos - MiningOS integration module
//!
//! This module provides bidirectional integration between BLVM and MiningOS,
//! enabling BLVM to register as a MiningOS "rack" (worker) via P2P and
//! query MiningOS via HTTP REST API.

pub mod actions;
pub mod api;
pub mod config;
pub mod data;
pub mod error;
pub mod http;
pub mod manager;
pub mod module;
pub mod p2p;

pub use config::MiningOsConfig;
pub use error::{MiningOsError, Result};
pub use manager::MiningOsIntegrationManager;
pub use module::MiningOsModule;

// Re-export commonly used types
pub use actions::{ActionHandler, ActionResult};
pub use data::{BlockTemplateProvider, StatisticsCollector, ThingConverter};
pub use http::MiningOsHttpClient;
