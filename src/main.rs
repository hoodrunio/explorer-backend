use dotenvy::dotenv;
use server::start_web_server;

mod chain;
mod cron_jobs;
mod database;
mod encoding;
mod events;
mod fetch;
mod logging;
mod macros;
mod routes;
mod server;
mod state;
mod utils;

use logging::{LogLevel::*, log_api};

#[tokio::main]
async fn main() {
    let _ = dotenv();
    
    // Initialize file logging
    log_api(INFO, "Application starting...");
    
    match start_web_server().await {
        Ok(_) => {
            log_api(INFO, "Web server started successfully");
            log_api(INFO, "Application running...");
        }
        Err(e) => {
            log_api(ERROR, &format!("Failed to start web server: {}", e));
        }
    }
    
    log_api(INFO, "Application stopped");
}
