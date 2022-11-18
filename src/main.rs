use server::start_web_server;

mod chain;
mod data;
mod encoding;
mod fetch;
mod macros;
mod routes;
mod server;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    println!("Server is running...");

    start_web_server().await.unwrap();
}
