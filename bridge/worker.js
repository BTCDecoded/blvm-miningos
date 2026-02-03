#!/usr/bin/env node
'use strict'

/**
 * Node.js bridge worker for MiningOS P2P integration
 * 
 * This worker bridges Hyperswarm P2P (Node.js-only) with the Rust blvm-miningos module.
 * It extends TetherWrkBase to integrate with MiningOS orchestrator and communicates
 * with the Rust module via Unix socket IPC using JSON-RPC protocol.
 */

const TetherWrkBase = require('tether-wrk-base/workers/base.wrk.tether')
const { RustIpcClient } = require('./ipc-client')

class BlvmMiningosBridge extends TetherWrkBase {
    constructor(conf, ctx) {
        super(conf, ctx)
        
        // IPC client for Rust module
        const socketPath = process.env.BLVM_RUST_SOCKET_PATH || './data/bridge.sock'
        this.rustIpc = new RustIpcClient(socketPath)
        
        // Rack configuration
        this.rackId = process.env.BLVM_RACK_ID || 'blvm-node-001'
        this.rackType = 'miner'
        
        // Initialize
        this.init()
    }

    init() {
        super.init()
        
        // Register RPC handlers for MiningOS orchestrator
        // These methods will be called by the orchestrator via Hyperswarm RPC
        
        this.net_r0.rpcServer.respond('listThings', async (req) => {
            this.debugGeneric('RPC: listThings', req)
            try {
                if (!this.rustIpc.isConnected()) {
                    await this.rustIpc.connect()
                }
                const result = await this.rustIpc.call('listThings', req || {})
                return result
            } catch (err) {
                this.debugError('listThings failed', err)
                throw err
            }
        })

        this.net_r0.rpcServer.respond('getBlockTemplate', async (req) => {
            this.debugGeneric('RPC: getBlockTemplate', req)
            try {
                if (!this.rustIpc.isConnected()) {
                    await this.rustIpc.connect()
                }
                const result = await this.rustIpc.call('getBlockTemplate', req || {})
                return result
            } catch (err) {
                this.debugError('getBlockTemplate failed', err)
                throw err
            }
        })

        this.net_r0.rpcServer.respond('executeAction', async (req) => {
            this.debugGeneric('RPC: executeAction', req)
            try {
                if (!this.rustIpc.isConnected()) {
                    await this.rustIpc.connect()
                }
                
                // Extract action type and params from request
                const actionType = req.action || req.action_type
                const params = req.params || {}
                
                const result = await this.rustIpc.call('executeAction', {
                    action: actionType,
                    params: params
                })
                return result
            } catch (err) {
                this.debugError('executeAction failed', err)
                throw err
            }
        })

        this.net_r0.rpcServer.respond('tailLog', async (req) => {
            this.debugGeneric('RPC: tailLog', req)
            try {
                // For now, return empty log - can be enhanced to forward to Rust
                return []
            } catch (err) {
                this.debugError('tailLog failed', err)
                throw err
            }
        })

        this.net_r0.rpcServer.respond('ping', async (req) => {
            try {
                if (!this.rustIpc.isConnected()) {
                    await this.rustIpc.connect()
                }
                const result = await this.rustIpc.call('ping', {})
                return { status: 'ok', ...result }
            } catch (err) {
                return { status: 'error', error: err.message }
            }
        })

        this.debugGeneric('Registered RPC handlers for MiningOS orchestrator')
    }

    async _start(cb) {
        try {
            // Connect to Rust module
            this.debugGeneric('Connecting to Rust IPC server...')
            await this.rustIpc.connect()
            this.debugGeneric('Connected to Rust IPC server')

            // Call parent start
            await super._start(cb)

            // Register this rack with MiningOS orchestrator
            // TetherWrkBase handles the Hyperswarm connection and registration
            this.debugGeneric(`Rack ${this.rackId} started and registered with MiningOS`)
        } catch (err) {
            this.debugError('Failed to start bridge', err)
            if (cb) cb(err)
            return
        }

        if (cb) cb(null)
    }

    async _stop(cb) {
        this.debugGeneric('Stopping bridge...')

        try {
            // Disconnect from Rust module
            await this.rustIpc.disconnect()
        } catch (err) {
            this.debugError('Error disconnecting from Rust module', err)
        }

        // Call parent stop
        await super._stop(cb)

        if (cb) cb(null)
    }

    /**
     * Handle periodic tasks (called by TetherWrkBase)
     */
    async _periodic() {
        // Check Rust IPC connection health
        if (!this.rustIpc.isConnected()) {
            this.debugGeneric('Rust IPC disconnected, attempting reconnect...')
            try {
                await this.rustIpc.connect()
            } catch (err) {
                this.debugError('Failed to reconnect to Rust IPC', err)
            }
        }

        // Call parent periodic
        if (super._periodic) {
            await super._periodic()
        }
    }
}

// Export for use by TetherWrkBase
module.exports = BlvmMiningosBridge

// If run directly, start the worker
if (require.main === module) {
    const worker = new BlvmMiningosBridge({}, {})
    worker.start((err) => {
        if (err) {
            console.error('Failed to start worker:', err)
            process.exit(1)
        }
    })

    // Handle shutdown
    process.on('SIGINT', () => {
        console.log('Shutting down...')
        worker.stop(() => {
            process.exit(0)
        })
    })

    process.on('SIGTERM', () => {
        console.log('Shutting down...')
        worker.stop(() => {
            process.exit(0)
        })
    })
}
