use dotenvy::dotenv;
use server::start_web_server;

mod chain;
mod encoding;
mod fetch;
mod macros;
mod routes;
mod server;
mod state;
mod utils;
mod database;
mod cron_jobs;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting...");
    start_web_server().await.unwrap();
    tracing::info!("Stopped!");
}
