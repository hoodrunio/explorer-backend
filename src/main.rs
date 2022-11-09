use crate::server::start_web_server;

mod chains;
mod fetch;
mod macros;
mod server;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    start_web_server().await.unwrap()
}
