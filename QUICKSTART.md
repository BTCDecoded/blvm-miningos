# blvm-miningos Quick Start Guide

## Prerequisites

- BLVM node running
- Node.js 16+ (for bridge worker)
- OAuth2 credentials (for HTTP API access)

## Installation

### 1. Build the Module

```bash
cd blvm-miningos
cargo build --release
```

### 2. Install Node.js Bridge Dependencies

```bash
cd bridge
npm install
cd ..
```

## Configuration

### 1. Create Configuration File

Create `data/config/miningos.toml`:

```toml
[miningos]
enabled = true

[p2p]
enabled = true
rack_id = "blvm-node-001"
rack_type = "miner"
auto_register = true

[http]
enabled = true
app_node_url = "https://api.mos.tether.io"
oauth_provider = "google"
oauth_client_id = "your-client-id"
oauth_client_secret = "your-client-secret"
oauth_callback_url = "http://localhost:8080/callback"
token_cache_file = "miningos-token.json"

[stats]
enabled = true
collection_interval_seconds = 60

[template]
enabled = true
update_interval_seconds = 30
```

### 2. Set Environment Variables (Optional)

```bash
# For OAuth token (if not using OAuth2 flow)
export MININGOS_ACCESS_TOKEN="your-oauth-token"

# For custom OAuth endpoint
export MININGOS_OAUTH_TOKEN_URL="https://custom-oauth-endpoint.com/token"

# For bridge communication
export BLVM_RUST_SOCKET_PATH="./data/bridge.sock"
export BLVM_RACK_ID="blvm-node-001"
```

## Running

### Automatic (via BLVM Node)

The module will be automatically discovered and loaded when placed in the modules directory:

```bash
# Module should be at: modules/blvm-miningos/
# BLVM node will auto-discover and load it
```

### Manual Testing

```bash
./target/release/blvm-miningos \
    --module-id blvm-miningos \
    --socket-path ./data/modules/modules.sock \
    --data-dir ./data
```

## Verification

### Check Module Status

1. Check BLVM node logs for module loading:
   ```
   INFO Loading module: blvm-miningos
   INFO Module blvm-miningos loaded successfully
   ```

2. Check bridge worker (if P2P enabled):
   ```bash
   ps aux | grep "node.*worker.js"
   ```

3. Check IPC socket:
   ```bash
   ls -la ./data/bridge.sock
   ```

### Test HTTP API (if enabled)

```bash
# List things
curl -H "Authorization: Bearer $MININGOS_ACCESS_TOKEN" \
  https://api.mos.tether.io/auth/list-things
```

### Test P2P Bridge

The bridge worker should automatically:
- Connect to Rust module via Unix socket
- Register with MiningOS orchestrator via Hyperswarm
- Respond to RPC calls from orchestrator

## Troubleshooting

### Module Not Loading

- Check `module.toml` exists and is valid
- Verify binary exists: `ls -la target/release/blvm-miningos`
- Check BLVM node logs for errors

### Bridge Worker Not Starting

- Verify Node.js installed: `node --version`
- Check dependencies: `cd bridge && npm list`
- Verify `bridge/worker.js` exists
- Check environment variables: `echo $BLVM_RUST_SOCKET_PATH`

### OAuth Authentication Failing

- Verify token in cache: `cat data/miningos-token.json`
- Check OAuth credentials in config
- Try setting `MININGOS_ACCESS_TOKEN` env var
- Check HTTP client logs

### P2P Connection Issues

- Verify Hyperswarm is working (check bridge logs)
- Check orchestrator topic configuration
- Verify rack_id is unique
- Check network connectivity

## Next Steps

1. **Integration Testing**: Test with actual BLVM node and MiningOS orchestrator
2. **Stratum V2 Integration**: Load Stratum V2 module to enable pool config updates
3. **Action Testing**: Test action execution (pool config update should work)
4. **Statistics Monitoring**: Monitor statistics collection and updates

## Support

- See `docs/INTEGRATION.md` for detailed integration guide
- See `VALIDATION.md` for implementation status
- See `README.md` for architecture overview

