//! P2P worker integration (Node.js bridge)

pub mod bridge_server;
pub mod bridge_manager;

pub use bridge_server::BridgeIpcServer;
pub use bridge_manager::BridgeManager;


