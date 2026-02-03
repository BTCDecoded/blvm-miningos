# Changelog

All notable changes to the blvm-miningos module will be documented in this file.

## [0.1.0] - 2026-02-03

### Added
- Initial implementation of MiningOS integration module
- HTTP REST API client with OAuth2 authentication
- OAuth2 token refresh flow (Google and custom providers)
- P2P bridge worker (Node.js) for Hyperswarm integration
- Block template provider with caching
- Statistics collector with chain info, network stats, and mempool data
- Action execution system with UpdatePoolConfig executor (Stratum V2 integration)
- Data conversion layer (BLVM → MiningOS "Thing" format)
- Hashrate estimation from difficulty
- IPC communication (Unix socket, JSON-RPC)
- Event subscription and handling
- Configuration system (TOML-based)
- Comprehensive error handling

### Implementation Status
- ✅ Core architecture complete
- ✅ OAuth2 token refresh implemented
- ✅ Pool config executor with Stratum V2 integration
- ✅ Enhanced statistics collection
- ⚠️ Reboot, SetPowerMode, SetHashrate executors (stubs)
- ⚠️ Real mining statistics (requires Stratum V2 module)

### Documentation
- README.md with architecture overview
- Integration guide (docs/INTEGRATION.md)
- Validation report (VALIDATION.md)
- Bridge documentation (bridge/README.md)
- Usage example (examples/basic_usage.rs)

