//! blvm-miningos - MiningOS integration module
//!
//! This module provides bidirectional integration between BLVM and MiningOS,
//! enabling BLVM to register as a MiningOS "rack" (worker) via P2P and
//! query MiningOS via HTTP REST API.

pub mod http;
pub mod data;
pub mod actions;
pub mod p2p;
pub mod manager;
pub mod config;
pub mod error;
pub mod client;
pub mod nodeapi_ipc;

pub use manager::MiningOsIntegrationManager;
pub use config::MiningOsConfig;
pub use error::{MiningOsError, Result};

// Re-export commonly used types
pub use http::MiningOsHttpClient;
pub use data::{ThingConverter, StatisticsCollector, BlockTemplateProvider};
pub use actions::{ActionHandler, ActionResult};

