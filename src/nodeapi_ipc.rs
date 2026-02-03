//! NodeAPI IPC wrapper for MiningOS module

use blvm_node::module::ipc::client::ModuleIpcClient;
use blvm_node::module::ipc::protocol::{RequestMessage, RequestPayload, ResponsePayload};
use blvm_node::module::traits::{ModuleError, NodeAPI};
use blvm_protocol::{Block, BlockHeader, Hash, OutPoint, Transaction, UTXO};
use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;

/// NodeAPI implementation over IPC
pub struct NodeApiIpc {
    ipc_client: Arc<Mutex<ModuleIpcClient>>,
    correlation_id: Arc<Mutex<u64>>,
}

impl NodeApiIpc {
    /// Create a new NodeAPI IPC wrapper
    pub fn new(ipc_client: Arc<Mutex<ModuleIpcClient>>) -> Self {
        Self {
            ipc_client,
            correlation_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Get next correlation ID
    async fn next_correlation_id(&self) -> u64 {
        let mut id = self.correlation_id.lock().await;
        *id += 1;
        *id
    }

    /// Helper method to make IPC requests
    async fn request<T, F>(
        &self,
        payload: RequestPayload,
        mapper: F,
    ) -> Result<T, ModuleError>
    where
        F: FnOnce(ResponsePayload) -> Result<T, ModuleError>,
    {
        let correlation_id = self.next_correlation_id().await;
        
        // Determine message type from payload
        let request_type = match &payload {
            RequestPayload::GetBlock { .. } => blvm_node::module::ipc::protocol::MessageType::GetBlock,
            RequestPayload::GetBlockHeader { .. } => blvm_node::module::ipc::protocol::MessageType::GetBlockHeader,
            RequestPayload::GetTransaction { .. } => blvm_node::module::ipc::protocol::MessageType::GetTransaction,
            RequestPayload::HasTransaction { .. } => blvm_node::module::ipc::protocol::MessageType::HasTransaction,
            RequestPayload::GetChainTip => blvm_node::module::ipc::protocol::MessageType::GetChainTip,
            RequestPayload::GetBlockHeight => blvm_node::module::ipc::protocol::MessageType::GetBlockHeight,
            RequestPayload::GetUtxo { .. } => blvm_node::module::ipc::protocol::MessageType::GetUtxo,
            RequestPayload::GetBlockTemplate { .. } => blvm_node::module::ipc::protocol::MessageType::GetBlockTemplate,
            _ => return Err(ModuleError::OperationError("Unsupported request payload".to_string())),
        };
        
        let request = RequestMessage {
            correlation_id,
            request_type,
            payload,
        };

        let response = self.ipc_client.lock().await.request(request).await?;

        if !response.success {
            return Err(ModuleError::OperationError(
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        match response.payload {
            Some(payload) => mapper(payload),
            None => Err(ModuleError::OperationError("Empty response payload".to_string())),
        }
    }
}

#[async_trait]
impl NodeAPI for NodeApiIpc {
    async fn get_block(&self, hash: &Hash) -> Result<Option<Block>, ModuleError> {
        self.request(
            RequestPayload::GetBlock { hash: *hash },
            |payload| match payload {
                ResponsePayload::Block(block) => Ok(block),
                _ => Ok(None),
            },
        )
        .await
    }

    async fn get_block_header(&self, hash: &Hash) -> Result<Option<BlockHeader>, ModuleError> {
        self.request(
            RequestPayload::GetBlockHeader { hash: *hash },
            |payload| match payload {
                ResponsePayload::BlockHeader(header) => Ok(header),
                _ => Ok(None),
            },
        )
        .await
    }

    async fn get_transaction(&self, hash: &Hash) -> Result<Option<Transaction>, ModuleError> {
        self.request(
            RequestPayload::GetTransaction { hash: *hash },
            |payload| match payload {
                ResponsePayload::Transaction(tx) => Ok(tx),
                _ => Ok(None),
            },
        )
        .await
    }

    async fn has_transaction(&self, hash: &Hash) -> Result<bool, ModuleError> {
        self.request(
            RequestPayload::HasTransaction { hash: *hash },
            |payload| match payload {
                ResponsePayload::Bool(has) => Ok(has),
                _ => Ok(false),
            },
        )
        .await
    }

    async fn get_chain_tip(&self) -> Result<Hash, ModuleError> {
        self.request(
            RequestPayload::GetChainTip,
            |payload| match payload {
                ResponsePayload::Hash(hash) => Ok(hash),
                _ => Err(ModuleError::OperationError("Invalid response".to_string())),
            },
        )
        .await
    }

    async fn get_block_height(&self) -> Result<u64, ModuleError> {
        self.request(
            RequestPayload::GetBlockHeight,
            |payload| match payload {
                ResponsePayload::U64(height) => Ok(height),
                _ => Err(ModuleError::OperationError("Invalid response".to_string())),
            },
        )
        .await
    }

    async fn get_utxo(&self, outpoint: &OutPoint) -> Result<Option<UTXO>, ModuleError> {
        self.request(
            RequestPayload::GetUtxo { outpoint: outpoint.clone() },
            |payload| match payload {
                ResponsePayload::Utxo(utxo) => Ok(utxo),
                _ => Ok(None),
            },
        )
        .await
    }

    async fn subscribe_events(
        &self,
        _event_types: Vec<blvm_node::module::traits::EventType>,
    ) -> Result<tokio::sync::mpsc::Receiver<blvm_node::module::ipc::protocol::ModuleMessage>, ModuleError> {
        // Event subscription is handled by ModuleClient
        // This is a stub implementation
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        Ok(rx)
    }

    async fn get_mempool_transactions(&self) -> Result<Vec<Hash>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_mempool_transaction(&self, _tx_hash: &Hash) -> Result<Option<Transaction>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_mempool_size(&self) -> Result<blvm_node::module::traits::MempoolSize, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_network_stats(&self) -> Result<blvm_node::module::traits::NetworkStats, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_network_peers(&self) -> Result<Vec<blvm_node::module::traits::PeerInfo>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_chain_info(&self) -> Result<blvm_node::module::traits::ChainInfo, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_block_by_height(&self, _height: u64) -> Result<Option<Block>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_lightning_node_url(&self) -> Result<Option<String>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_lightning_info(&self) -> Result<Option<blvm_node::module::traits::LightningInfo>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_payment_state(&self, _payment_id: &str) -> Result<Option<blvm_node::module::traits::PaymentState>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn check_transaction_in_mempool(&self, _tx_hash: &Hash) -> Result<bool, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_fee_estimate(&self, _target_blocks: u32) -> Result<u64, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn register_rpc_endpoint(&self, _method: String, _description: String) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn unregister_rpc_endpoint(&self, _method: &str) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn register_timer(
        &self,
        _interval_seconds: u64,
        _callback: Arc<dyn blvm_node::module::timers::manager::TimerCallback>,
    ) -> Result<blvm_node::module::timers::manager::TimerId, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn cancel_timer(&self, _timer_id: blvm_node::module::timers::manager::TimerId) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn schedule_task(
        &self,
        _delay_seconds: u64,
        _callback: Arc<dyn blvm_node::module::timers::manager::TaskCallback>,
    ) -> Result<blvm_node::module::timers::manager::TaskId, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn report_metric(&self, _metric: blvm_node::module::metrics::manager::Metric) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_module_metrics(&self, _module_id: &str) -> Result<Vec<blvm_node::module::metrics::manager::Metric>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_all_metrics(
        &self,
    ) -> Result<std::collections::HashMap<String, Vec<blvm_node::module::metrics::manager::Metric>>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn read_file(&self, _path: String) -> Result<Vec<u8>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn write_file(&self, _path: String, _data: Vec<u8>) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn delete_file(&self, _path: String) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn list_directory(&self, _path: String) -> Result<Vec<String>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn create_directory(&self, _path: String) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_file_metadata(&self, _path: String) -> Result<blvm_node::module::ipc::protocol::FileMetadata, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_open_tree(&self, _name: String) -> Result<String, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_insert(&self, _tree_id: String, _key: Vec<u8>, _value: Vec<u8>) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_get(&self, _tree_id: String, _key: Vec<u8>) -> Result<Option<Vec<u8>>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_remove(&self, _tree_id: String, _key: Vec<u8>) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_contains_key(&self, _tree_id: String, _key: Vec<u8>) -> Result<bool, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_iter(&self, _tree_id: String) -> Result<Vec<(Vec<u8>, Vec<u8>)>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn storage_transaction(
        &self,
        _tree_id: String,
        _operations: Vec<blvm_node::module::ipc::protocol::StorageOperation>,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn initialize_module(
        &self,
        _module_id: String,
        _module_data_dir: std::path::PathBuf,
        _base_data_dir: std::path::PathBuf,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn discover_modules(&self) -> Result<Vec<blvm_node::module::traits::ModuleInfo>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_module_info(&self, _module_id: &str) -> Result<Option<blvm_node::module::traits::ModuleInfo>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn is_module_available(&self, _module_id: &str) -> Result<bool, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn publish_event(
        &self,
        _event_type: blvm_node::module::traits::EventType,
        _payload: blvm_node::module::ipc::protocol::EventPayload,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn call_module(
        &self,
        _target_module_id: Option<&str>,
        _method: &str,
        _params: Vec<u8>,
    ) -> Result<Vec<u8>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn register_module_api(
        &self,
        _api: Arc<dyn blvm_node::module::inter_module::api::ModuleAPI>,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn unregister_module_api(&self) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn send_mesh_packet_to_module(
        &self,
        _module_id: &str,
        _packet_data: Vec<u8>,
        _peer_addr: String,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn send_mesh_packet_to_peer(&self, _peer_addr: String, _packet_data: Vec<u8>) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn send_stratum_v2_message_to_peer(
        &self,
        _peer_addr: String,
        _message: Vec<u8>,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_module_health(
        &self,
        _module_id: &str,
    ) -> Result<Option<blvm_node::module::process::monitor::ModuleHealth>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_all_module_health(
        &self,
    ) -> Result<Vec<(String, blvm_node::module::process::monitor::ModuleHealth)>, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn report_module_health(
        &self,
        _health: blvm_node::module::process::monitor::ModuleHealth,
    ) -> Result<(), ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }

    async fn get_block_template(
        &self,
        rules: Vec<String>,
        coinbase_script: Option<Vec<u8>>,
        coinbase_address: Option<String>,
    ) -> Result<blvm_protocol::mining::BlockTemplate, ModuleError> {
        self.request(
            RequestPayload::GetBlockTemplate {
                rules,
                coinbase_script,
                coinbase_address,
            },
            |payload| match payload {
                ResponsePayload::BlockTemplate(template) => Ok(template),
                _ => Err(ModuleError::OperationError("Unexpected response type".to_string())),
            },
        )
        .await
    }

    async fn submit_block(&self, _block: Block) -> Result<blvm_node::module::traits::SubmitBlockResult, ModuleError> {
        Err(ModuleError::OperationError("Not implemented".to_string()))
    }
}
