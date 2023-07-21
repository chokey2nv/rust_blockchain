use super::blockchain::chain::{Blockchain, Transaction};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::error::Error;

// Define the Transaction, NodePeer, and Blockchain structs as before

// Application represents the blockchain application.
pub struct Application {
    blockchain: Blockchain,
}

impl Application {
    // Create a new blockchain application.
    pub fn new() -> Result<Application, Box<dyn Error>> {
        let blockchain = Blockchain::new_blockchain()?;
        Ok(Application { blockchain })
    }

    // Define your handler functions here using Actix-web's syntax
    async fn new_transaction(transaction: web::Json<Transaction>) -> impl Responder {
        // Your new transaction handler logic
        HttpResponse::Ok().body("Hello, Actix!")
    }

    pub async fn get_chain(app: web::Data<Application>) -> impl Responder {
        // Your get chain handler logic
        HttpResponse::Ok().json(app.blockchain.chain.clone())
    }

    // Add more handler functions as needed

    // Setup Actix-web App and routes
    pub fn configure_routes(app: &mut web::ServiceConfig) {
        app.route("/new_transaction", web::post().to(Self::new_transaction));
        // app.route("/chain", web::get().to(Self::get_chain));
        // Add more routes here
    }
    fn config(cfg: &mut web::ServiceConfig) {
        let state = Application::new();
        cfg.app_data(state)
            .route("/chains", web::get().to(Application::get_chain));
    }
}

// Define a function to start the Actix-web server
pub async fn start_server() {
    match HttpServer::new(move || App::new().configure(Application::config)).bind("127.0.0.1:8080")
    {
        Ok(server) => {
            println!("Server started at: http://127.0.0.1:8080");
            if let Err(e) = server.run().await {
                eprintln!("Error running server: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error binding to address: {}", e);
        }
    }
}
