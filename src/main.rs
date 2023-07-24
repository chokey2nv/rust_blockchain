mod modules;

use std::error::Error;

use actix_web::{web, App, HttpServer, Responder};
use tokio::{select, signal::ctrl_c};

use crate::modules::{
    app::start_node,
    client::{start_client, Client},
};

async fn hello() -> impl Responder {
    "Hello, World!"
}
// In your main.rs, you can now start the server as follows:
#[tokio::main]
async fn main() {
    HttpServer::new(move || App::new().route("/", web::get().to(Client::handle_index)))
        .bind("127.0.0.1:8000")
        .unwrap()
        .run()
        .await;
    start_client().await;
    let _ = HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
        .bind("127.0.0.1:8080")
        .unwrap()
        .run()
        .await;
    // let client_server = tokio::spawn(start_client().await);
    // let node_server = tokio::spawn(start_node().await);

    // // Wait for both servers to complete or for a termination signal
    // select! {
    //     _ = client_server => {},
    //     _ = node_server => {},
    //     _ = ctrl_c() => {},
    // }

    // Print success message
    println!("Both servers started successfully");
}
