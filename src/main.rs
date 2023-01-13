use dotenvy::dotenv;
use server::start_web_server;

mod chain;
mod cron_jobs;
mod database;
mod encoding;
mod events;
mod fetch;
mod macros;
mod routes;
mod server;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting...");
    start_web_server().await.unwrap();
    tracing::info!("Stopped!");
}
