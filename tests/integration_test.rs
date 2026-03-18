//! Integration tests for blvm-miningos module

#[cfg(test)]
mod tests {
    use blvm_miningos::*;
    
    #[tokio::test]
    async fn test_config_loading() {
        // Test that default config works
        let config = blvm_miningos::MiningOsConfig::default();
        assert!(config.miningos.enabled);
        assert!(config.p2p.is_some());
        assert!(config.http.is_some());
    }
    
    #[tokio::test]
    async fn test_error_types() {
        // Test error creation
        let err = blvm_miningos::MiningOsError::AuthError("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}


