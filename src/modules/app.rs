use super::blockchain::{
    self,
    block::Block,
    chain::{Blockchain, NodePeer, Transaction},
};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde_json::{from_value, json, Map, Value};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

// Define the Transaction, NodePeer, and Blockchain structs as before

// Application represents the blockchain application.
pub struct Application {
    pub blockchain: Mutex<Blockchain>,
}

impl Application {
    // Create a new blockchain application.
    pub fn new() -> Result<Application, Box<dyn Error>> {
        let blockchain = Blockchain::new_blockchain()?;
        Ok(Application {
            blockchain: Mutex::new(blockchain),
        })
    }
    // Endpoint /register_with handler function - registers node to list via synced node and syncs the calling node
    pub async fn handle_register_node_with(
        node: web::Json<NodePeer>,
        req: HttpRequest,
    ) -> HttpResponse {
        let node = node.into_inner();

        // handle empty node address
        if node.node_address.is_empty() {
            return HttpResponse::BadRequest().body("Invalid node data");
        }

        // Prepare the request payload
        let data = json!({
            "node_address": req.connection_info().host(),
        });
        let payload = data.to_string();

        // Make a request to register with the remote node
        let client = reqwest::Client::new();
        let response = match client
            .post(format!("{}/register_node", node.node_address))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => return HttpResponse::InternalServerError().body("Internal Server Error"),
        };

        let status = response.status();
        if status == reqwest::StatusCode::OK {
            let body = response.text().await.unwrap();
            let parsed_response: serde_json::Map<String, Value> =
                serde_json::from_str(&body).unwrap();

            let chain = parsed_response
                .get("chain")
                .and_then(|c| c.as_array())
                .map(|array| {
                    array
                        .iter()
                        .filter_map(|value| from_value::<Map<String, Value>>(value.clone()).ok())
                        .collect::<Vec<Map<String, Value>>>()
                })
                .expect("Unable to unbox received chain data");
            let peers = parsed_response
                .get("peers")
                .and_then(|p| p.as_array())
                .map(|array| {
                    array
                        .iter()
                        .filter_map(|value| from_value::<String>(value.clone()).ok())
                        .collect::<Vec<String>>()
                })
                .expect("Unable to unbox received peers");

            match Blockchain::create_chain_from_dump(chain, peers) {
                Ok(_) => return HttpResponse::Ok().body("Registration successful"),
                Err(message) => return HttpResponse::Conflict().body(message.to_string()),
            };
        } else {
            let body = response.text().await.unwrap();
            HttpResponse::build(status).body(body)
        }
    }

    // Endpoint /register_node handler - adds node peer to list
    pub async fn handle_register_node(
        blockchain: web::Data<Mutex<Blockchain>>,
        req: web::Json<NodePeer>,
    ) -> HttpResponse {
        let node = req.into_inner();

        // Check and prevent empty node_address
        if node.node_address.is_empty() {
            return HttpResponse::BadRequest().body("Invalid node data");
        }
        let mut blockchain = blockchain
            .lock()
            .expect("Unable to lock blockchain data for update");
        // Add peer to list
        blockchain.add_node_peer(node);

        let response_data = json!({
            "chain": blockchain.chain,
            "peers": blockchain.peers,
        });

        let response_str = serde_json::to_string(&response_data).unwrap();
        HttpResponse::Created().body(response_str)
    }
    pub async fn handle_get_pending_transactions(
        blockchain: web::Data<Mutex<Blockchain>>,
    ) -> impl Responder {
        let mut blockchain = blockchain
            .lock()
            .expect("Unable to block blockchain for update");

        let response_json =
            serde_json::to_string(&blockchain.unconfirmed_transactions).map_err(|e| {
                eprintln!("Error marshaling pending transaction data: {}", e);
                HttpResponse::InternalServerError()
            });

        match response_json {
            Ok(json) => HttpResponse::Ok().json(json),
            Err(response) => response.into(),
        }
    }
    pub async fn handle_new_transaction(
        transaction: web::Json<Transaction>,
        blockchain: web::Data<Mutex<Blockchain>>,
    ) -> impl Responder {
        // Lock the Mutex to gain access to the Arc
        let mut arc_blockchain = blockchain
            .lock()
            .expect("Unable to block blockchain for update");
        let mut transaction_data = transaction.into_inner();

        // Validate transaction details
        if transaction_data.author.is_empty() || transaction_data.content.is_empty() {
            return HttpResponse::BadRequest().body("Invalid transaction data");
        }
        let current_timestamp = chrono::Utc::now().timestamp();
        transaction_data.timestamp = current_timestamp;

        // Add new tx to pending tx (unconfirmed transactions)
        arc_blockchain.add_new_transaction(transaction_data);

        HttpResponse::Created().body("Success")
    }
    pub async fn get_chain(blchain: web::Data<Blockchain>) -> impl Responder {
        HttpResponse::Ok().json(blchain.chain.clone())
    }

    fn config(cfg: &mut web::ServiceConfig) {
        let app = Application::new().expect("Failed to initialize application");

        cfg.app_data(app.blockchain)
            .service(web::resource("/chains").route(web::post().to(Self::get_chain)))
            .service(
                web::resource("/new_transaction")
                    .route(web::post().to(Self::handle_new_transaction)),
            )
            .service(
                web::resource("/mempool")
                    .route(web::post().to(Self::handle_get_pending_transactions)),
            )
            .service(
                web::resource("/register_with")
                    .route(web::post().to(Self::handle_register_node_with)),
            )
            .service(
                web::resource("/register_node").route(web::post().to(Self::handle_register_node)),
            );
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
