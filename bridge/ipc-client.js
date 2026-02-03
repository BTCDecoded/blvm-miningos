//! IPC client for communicating with Rust module via Unix socket

const net = require('net');

/**
 * IPC client for Rust module communication
 * Uses JSON-RPC over Unix domain socket
 */
class RustIpcClient {
    constructor(socketPath) {
        this.socketPath = socketPath;
        this.socket = null;
        this.connected = false;
        this.callbacks = new Map();
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000; // 1 second
    }

    /**
     * Connect to Rust module
     */
    async connect() {
        return new Promise((resolve, reject) => {
            if (this.connected && this.socket) {
                resolve();
                return;
            }

            console.log(`[IPC] Connecting to Rust module at ${this.socketPath}`);

            this.socket = net.createConnection(this.socketPath);

            let buffer = '';

            this.socket.on('connect', () => {
                console.log('[IPC] Connected to Rust module');
                this.connected = true;
                this.reconnectAttempts = 0;
                resolve();
            });

            this.socket.on('error', (err) => {
                console.error('[IPC] Socket error:', err.message);
                this.connected = false;
                
                if (err.code === 'ENOENT' || err.code === 'ECONNREFUSED') {
                    // Socket doesn't exist yet or connection refused - try to reconnect
                    this.scheduleReconnect();
                }
                
                if (this.reconnectAttempts === 0) {
                    reject(err);
                }
            });

            this.socket.on('close', () => {
                console.log('[IPC] Socket closed');
                this.connected = false;
                this.scheduleReconnect();
            });

            this.socket.on('data', (data) => {
                buffer += data.toString();
                const lines = buffer.split('\n');
                buffer = lines.pop() || ''; // Keep incomplete line in buffer

                for (const line of lines) {
                    if (line.trim()) {
                        try {
                            const response = JSON.parse(line);
                            this.handleResponse(response);
                        } catch (err) {
                            console.error('[IPC] Failed to parse response:', err.message, line);
                        }
                    }
                }
            });
        });
    }

    /**
     * Schedule reconnection attempt
     */
    scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('[IPC] Max reconnection attempts reached');
            return;
        }

        this.reconnectAttempts++;
        const delay = this.reconnectDelay * this.reconnectAttempts;

        console.log(`[IPC] Scheduling reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

        setTimeout(() => {
            if (!this.connected) {
                this.connect().catch(err => {
                    console.error('[IPC] Reconnection failed:', err.message);
                });
            }
        }, delay);
    }

    /**
     * Handle response from Rust module
     */
    handleResponse(response) {
        if (response.id && this.callbacks.has(response.id)) {
            const { resolve, reject, timeout } = this.callbacks.get(response.id);
            this.callbacks.delete(response.id);
            clearTimeout(timeout);

            if (response.error) {
                reject(new Error(response.error.message || 'RPC error'));
            } else {
                resolve(response.result);
            }
        }
    }

    /**
     * Call Rust module method via JSON-RPC
     * @param {string} method - RPC method name
     * @param {object} params - Method parameters
     * @returns {Promise<any>} - RPC result
     */
    async call(method, params = {}) {
        if (!this.connected || !this.socket) {
            throw new Error('Not connected to Rust module');
        }

        return new Promise((resolve, reject) => {
            const id = Date.now() + Math.random();
            const request = {
                jsonrpc: '2.0',
                id,
                method,
                params
            };

            // Set timeout (30 seconds)
            const timeout = setTimeout(() => {
                if (this.callbacks.has(id)) {
                    this.callbacks.delete(id);
                    reject(new Error(`RPC timeout for method: ${method}`));
                }
            }, 30000);

            this.callbacks.set(id, { resolve, reject, timeout });

            try {
                this.socket.write(JSON.stringify(request) + '\n');
            } catch (err) {
                this.callbacks.delete(id);
                clearTimeout(timeout);
                reject(err);
            }
        });
    }

    /**
     * Disconnect from Rust module
     */
    async disconnect() {
        if (this.socket) {
            this.socket.end();
            this.socket = null;
        }
        this.connected = false;
        this.callbacks.clear();
        console.log('[IPC] Disconnected from Rust module');
    }

    /**
     * Check if connected
     */
    isConnected() {
        return this.connected && this.socket !== null;
    }
}

module.exports = { RustIpcClient };

