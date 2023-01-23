use dotenvy::dotenv;
use server::start_web_server;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::util::SubscriberInitExt;

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
    let _guard = if let Ok(filename) = std::env::var("LOG_FILE") {
        let directory = std::env::var("LOG_DIRECTORY").unwrap_or_else(|e| ".".to_string());
        let file_appender = tracing_appender::rolling::daily(&directory, filename);

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_writer(non_blocking)
            .init();
        Some(_guard)
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        None
    };

    tracing::info!("Starting...");
    start_web_server().await.unwrap();
    tracing::info!("Stopped!");
}
