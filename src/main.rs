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
    println!("Server is running...");

    start_web_server().await.unwrap();
}
