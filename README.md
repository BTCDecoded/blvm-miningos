# blvm-miningos

MiningOS integration module for BLVM (Bitcoin Low-Level Virtual Machine).

This module provides bidirectional integration between BLVM and MiningOS (Mos), enabling:

- **BLVM → MiningOS**: Register BLVM as a MiningOS "rack" (worker), expose miners as "things", provide block templates
- **MiningOS → BLVM**: Execute actions (reboot, power management, etc.), query statistics, receive commands

## Architecture

The module uses a hybrid approach combining:

1. **Rust Module**: Core integration logic, HTTP client, data conversion, action handling
2. **Node.js Bridge**: P2P worker that extends `TetherWrkBase` for Hyperswarm integration
3. **IPC Communication**: Unix socket-based JSON-RPC between Rust and Node.js

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

## Features

- ✅ HTTP REST API client for MiningOS app-node
- ✅ **OAuth2 authentication with token refresh** (fully implemented)
- ✅ P2P worker bridge (Node.js) for Hyperswarm integration
- ✅ Block template provider
- ✅ **Enhanced statistics collection** (chain info, network stats, mempool)
- ✅ **Action execution system** (pool config update integrates with Stratum V2)
- ✅ Data conversion (BLVM → MiningOS "Thing" format)
- ✅ Event subscription (block mined, template updates, etc.)
- ✅ Hashrate estimation from difficulty

## Quick Start

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

## Building

```bash
# Build Rust module
cargo build --release

# Install Node.js bridge dependencies
cd bridge
npm install
```

## Configuration

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
token_cache_file = "miningos-token.json"

[stats]
enabled = true
collection_interval_seconds = 60

[template]
enabled = true
update_interval_seconds = 30
```

## Usage

The module is automatically discovered and loaded by the BLVM node system when placed in the modules directory.

For manual testing:

```bash
./target/release/blvm-miningos \
    --module-id blvm-miningos \
    --socket-path ./data/modules/modules.sock \
    --data-dir ./data
```

## Module Capabilities

The module requests the following capabilities:

- `read_blockchain` - Read blockchain data
- `subscribe_events` - Subscribe to node events
- `publish_events` - Publish events
- `call_module` - Call other modules
- `get_block_template` - Get block templates
- `submit_block` - Submit mined blocks

## Documentation

- [QUICKSTART.md](QUICKSTART.md) - Quick start guide
- [docs/INTEGRATION.md](docs/INTEGRATION.md) - Detailed integration guide
- [VALIDATION.md](VALIDATION.md) - Implementation validation report
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [bridge/README.md](bridge/README.md) - Bridge worker documentation

## License

MIT
