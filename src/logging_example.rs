use crate::logging::{LogLevel::*, log_db, log_api, log_tx, log_block, log_socket};

pub fn example_logging() {
    // Database related logs
    log_db(INFO, "Connected to MongoDB");
    log_db(ERROR, "Failed to execute query: Invalid syntax");
    
    // API related logs
    log_api(INFO, "Received request: GET /api/v1/status");
    log_api(WARN, "Rate limit exceeded for IP: 192.168.1.1");
    
    // Transaction related logs
    log_tx(INFO, "Processing transaction: 0x123...");
    log_tx(DEBUG, "Transaction details: {...}");
    
    // Block related logs
    log_block(INFO, "New block processed: 12345");
    log_block(ERROR, "Failed to process block: Invalid hash");
    
    // Socket related logs
    log_socket(INFO, "WebSocket connection established");
    log_socket(WARN, "Connection dropped: timeout");
    
    // General error logs (will be written to both category and error logs)
    //tracing::error!("Unexpected error occurred in the system");
}
