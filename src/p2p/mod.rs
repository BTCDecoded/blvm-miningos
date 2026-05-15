//! P2P worker integration (Node.js bridge)

pub mod bridge_manager;
pub mod bridge_server;

pub use bridge_manager::BridgeManager;
pub use bridge_server::BridgeIpcServer;
