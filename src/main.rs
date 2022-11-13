use crate::server::start_web_server;
mod chain;
mod data;
mod fetch;
mod routes;
mod server;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    println!("Server is starting...");
    start_web_server().await.unwrap()
}
