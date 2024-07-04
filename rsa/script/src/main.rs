mod handler;

use actix_web::{App, HttpServer};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = 3000;

    let server = HttpServer::new(move || App::new().configure(handler::routes))
        .client_request_timeout(Duration::new(0, 0))
        .bind(("0.0.0.0", port))
        .unwrap_or_else(|_| panic!("Can not bind to {}", &port))
        .run();

    server.await
}
