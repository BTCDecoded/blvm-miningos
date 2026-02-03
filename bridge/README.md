# Node.js Bridge Worker

This directory contains the Node.js bridge worker that connects the Rust `blvm-miningos` module with MiningOS's Hyperswarm P2P network.

## Overview

The bridge worker:
- Extends `TetherWrkBase` to integrate with MiningOS orchestrator
- Communicates with the Rust module via Unix socket IPC (JSON-RPC)
- Forwards RPC calls between MiningOS orchestrator and Rust module
- Handles Hyperswarm P2P networking (Node.js-only)

## Installation

```bash
npm install
```

## Configuration

The bridge is configured via environment variables:

- `BLVM_RUST_SOCKET_PATH`: Path to Unix socket for Rust IPC (required)
- `BLVM_RACK_ID`: Rack identifier for MiningOS (default: `blvm-node-001`)
- `MININGOS_ORCHESTRATOR_TOPIC`: Hyperswarm topic for orchestrator (optional)

## Usage

The bridge is automatically spawned by the Rust module when P2P is enabled. It can also be run manually for testing:

```bash
export BLVM_RUST_SOCKET_PATH=./data/bridge.sock
export BLVM_RACK_ID=blvm-node-001
node worker.js
```

## Architecture

```
MiningOS Orchestrator (Hyperswarm P2P)
           │
           │ RPC calls
           │
    ┌──────▼──────┐
    │ Node.js     │
    │ Bridge      │
    │ (worker.js) │
    └──────┬──────┘
           │ JSON-RPC over Unix socket
           │
    ┌──────▼──────┐
    │ Rust Module │
    │ (blvm-miningos)│
    └─────────────┘
```

## RPC Methods

The bridge forwards the following RPC methods:

- `listThings` - List mining things (miners, devices)
- `getBlockTemplate` - Get current block template
- `executeAction` - Execute MiningOS action
- `tailLog` - Get time-series log data
- `ping` - Health check

## Files

- `worker.js` - Main bridge worker (extends TetherWrkBase)
- `ipc-client.js` - IPC client for Rust communication
- `package.json` - Node.js dependencies

## Dependencies

- `tether-wrk-base` - MiningOS worker base class
- `hyperswarm` - P2P networking
- `@hyperswarm/rpc` - RPC over Hyperswarm

