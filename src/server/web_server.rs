use std::net::{Ipv4Addr, SocketAddrV4};

use actix_web::{get, web, App, HttpServer, Responder};

use super::state::ServerState;

/// Starts the web server.
pub async fn start_web_server() -> std::io::Result<()> {
    let socket_addr = ("127.0.0.1", 8080);

    HttpServer::new(move || App::new().app_data(web::Data::new(ServerState::new())))
        .bind(socket_addr)
        .expect(&format!("File: {}\nLine: {}", file!(), line!()))
        .run()
        .await
}
