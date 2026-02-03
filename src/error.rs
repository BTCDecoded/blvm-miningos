//! Error types for MiningOS integration module

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiningOsError {
    #[error("HTTP client error: {0}")]
    HttpError(String),
    
    #[error("HTTP request error: {0}")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("OAuth authentication failed: {0}")]
    AuthError(String),

    #[error("P2P connection failed: {0}")]
    P2PError(String),

    #[error("RPC call failed: {0}")]
    RpcError(String),

    #[error("Data conversion error: {0}")]
    ConversionError(String),

    #[error("Action execution failed: {0}")]
    ActionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IPC error: {0}")]
    IpcError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Module error: {0}")]
    ModuleError(#[from] blvm_node::module::traits::ModuleError),
}

pub type Result<T> = std::result::Result<T, MiningOsError>;

