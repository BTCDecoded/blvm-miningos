# MiningOS Integration Guide

This document describes how to integrate and use the `blvm-miningos` module with BLVM.

## Overview

The `blvm-miningos` module provides bidirectional integration between BLVM and MiningOS (Mos), enabling:

- **BLVM → MiningOS**: Register BLVM as a MiningOS "rack" (worker), expose miners as "things", provide block templates
- **MiningOS → BLVM**: Execute actions (reboot, power management, etc.), query statistics, receive commands

## Architecture

```
┌─────────────────────┐
│  MiningOS           │
│  Orchestrator       │
│  (Hyperswarm P2P)   │
└──────────┬──────────┘
           │
           │ Hyperswarm
           │
┌──────────▼──────────┐      Unix Socket      ┌──────────────┐
│  Node.js Bridge     │ ◄───────────────────► │ Rust Module  │
│  (worker.js)        │      JSON-RPC          │              │
└─────────────────────┘                        └──────┬───────┘
                                                       │ IPC
                                                       │
                                                ┌──────▼───────┐
                                                │  BLVM Node   │
                                                └──────────────┘
```

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
```

### 3. Configure the Module

Create configuration file at `data/config/miningos.toml`:

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
token_cache_file = "miningos-token.json"

[stats]
enabled = true
collection_interval_seconds = 60

[template]
enabled = true
update_interval_seconds = 30
```

### 4. Set Environment Variables (Optional)

```bash
export MININGOS_ACCESS_TOKEN="your-oauth-token-here"
export BLVM_RACK_ID="blvm-node-001"
```

## Usage

### Starting the Module

The module is typically started by the BLVM node as part of the module system. It can also be run standalone for testing:

```bash
./target/release/blvm-miningos \
    --module-id blvm-miningos \
    --socket-path ./data/modules/modules.sock \
    --data-dir ./data
```

### Module Capabilities

The module requests the following capabilities from the BLVM node:

- `read_blockchain` - Read blockchain data
- `subscribe_events` - Subscribe to node events
- `publish_events` - Publish events
- `call_module` - Call other modules
- `submit_block` - Submit mined blocks
- `get_block_template` - Get block templates

## API Reference

### HTTP REST API Methods

#### `list_things(query: ThingQuery) -> Vec<Thing>`

List miners/devices from MiningOS.

```rust
let things = http_client.list_things(&ThingQuery {
    query: Some(json!({"type": "miner"})),
    fields: None,
    sort: None,
    limit: Some(100),
}).await?;
```

#### `tail_log(params: TailLogParams) -> Vec<LogEntry>`

Get time-series log data.

```rust
let logs = http_client.tail_log(&TailLogParams {
    log_type: "mining".to_string(),
    key: Some("hashrate".to_string()),
    tag: Some("t-miner".to_string()),
    fields: Some(vec!["hashrate".to_string(), "temperature".to_string()]),
    start: None,
    end: None,
    limit: Some(1000),
}).await?;
```

#### `submit_action(action: Action) -> ActionId`

Submit an action for approval.

```rust
let action_id = http_client.submit_action(&Action {
    action_type: "reboot".to_string(),
    target: "blvm-node-001".to_string(),
    params: json!({}),
}).await?;
```

### P2P RPC Methods (via Bridge)

The Node.js bridge exposes the following RPC methods to MiningOS orchestrator:

- `listThings` - List things (forwarded to Rust)
- `getBlockTemplate` - Get block template (forwarded to Rust)
- `executeAction` - Execute action (forwarded to Rust)

### IPC Methods (Rust ↔ Node.js)

The Rust module exposes the following IPC methods to the Node.js bridge:

- `listThings` - Get list of mining things
- `getBlockTemplate` - Get current block template
- `executeAction` - Execute MiningOS action
- `ping` - Health check

## Event Handling

The module subscribes to the following BLVM node events:

- `BlockMined` - Triggers statistics collection
- `BlockTemplateUpdated` - Triggers template cache refresh
- `MiningDifficultyChanged` - Logs difficulty changes

## Troubleshooting

### Module Not Starting

1. Check that the module binary exists: `ls -la target/release/blvm-miningos`
2. Verify module.toml is in the module directory
3. Check BLVM node logs for module loading errors

### Bridge Worker Not Starting

1. Verify Node.js is installed: `node --version`
2. Check bridge dependencies: `cd bridge && npm list`
3. Verify bridge/worker.js exists and is executable
4. Check environment variables: `echo $BLVM_RUST_SOCKET_PATH`

### OAuth Authentication Failing

1. Verify OAuth credentials in config file
2. Check token cache file: `cat data/miningos-token.json`
3. Try setting `MININGOS_ACCESS_TOKEN` environment variable
4. Check HTTP client logs for authentication errors

### P2P Connection Issues

1. Verify Hyperswarm is working: Check Node.js bridge logs
2. Check orchestrator topic configuration
3. Verify rack_id is unique
4. Check network connectivity

## Development

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
cargo run --example basic_usage
```

### Building Documentation

```bash
cargo doc --open
```

## Security Considerations

1. **OAuth Tokens**: Store tokens securely, never commit to version control
2. **Bridge Communication**: Unix sockets are local-only, but ensure proper permissions
3. **P2P**: Hyperswarm connections are encrypted, but verify orchestrator public key
4. **Actions**: All actions require approval (multi-voter system) by default

## License

MIT


