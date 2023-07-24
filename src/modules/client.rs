use std::{error::Error, io, sync::Mutex, vec};

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize)]
pub struct Article {
    pub author: String,
    pub timestamp: i64,
    pub content: String,
}
// Define the base template
#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a> {
    title: &'a str,
}

// Define the index template
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    node_address: &'a str,
    articles: Vec<Article>,
    title: &'a str,
}
// Define a struct to hold the form data
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostObject {
    author: String,
    content: String,
}
pub struct Client {
    node: String,
}
impl Client {
    // Create a new blockchain application.
    pub fn new() -> Result<Client, Box<dyn Error>> {
        Ok(Client {
            node: "http://localhost:8080".to_owned(),
        })
    }
    fn get_node(&self) -> &str {
        &self.node
    }
    async fn handle_submit(
        form: web::Form<PostObject>,
        client: web::Data<Mutex<Client>>,
    ) -> HttpResponse {
        // Access the form values
        let author = &form.author;
        let content = &form.content;

        // Define node public address and path (method)
        let new_tx_address = client.lock().unwrap().get_node().to_string() + "/new_transaction"; // Replace with the actual node address

        // Serialize the form data to JSON
        let payload = serde_json::to_string(&json!({"author": author, "content": content}))
            .expect("Failed to serialize form data to JSON");

        // Post new transaction to node
        match reqwest::Client::new()
            .post(new_tx_address)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
        {
            Ok(_) => {
                // Redirect to home page
                HttpResponse::SeeOther()
                    .append_header(("Location", "/"))
                    .finish()
            }
            Err(err) => {
                eprintln!("Error posting new transaction: {}", err);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
    // Handler function to render the index template
    pub async fn handle_index() -> impl Responder {
        // let client: web::Data<Mutex<Client>>
        if true {
            return HttpResponse::Ok().body("hello");
        }
        // let client = client.lock().unwrap();
        let node_address: &str = ""; //client.get_node();
                                     // let get_chain_url = node_address.to_string() + "/chains";
        let title = "My Blog";
        /* let response = reqwest::Client::new()
            .get(get_chain_url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap();
        if !response.status().is_success() {
            return HttpResponse::InternalServerError().body("Failed to get chain data");
        }
        let body = json!(response.text().await.unwrap());
        let articles = match &body["chain"]["unconfirmed_transactions"] {
            Value::Array(array) => array
                .iter()
                .filter_map(|value| {
                    serde_json::from_value::<Article>(value.clone())
                        .map(|mut article| {
                            article.timestamp = chrono::Utc::now().timestamp();
                            article
                        })
                        .ok()
                })
                .collect(),
            _ => Vec::new(),
        }; */
        // Create an instance of the IndexTemplate with the data
        let articles = vec![Article {
            author: "Agu".to_string(),
            content: "Body".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }];

        let index_template = IndexTemplate {
            node_address,
            articles,
            title,
        };

        // Render the template
        let html = index_template.render().unwrap();
        // Return the HTML content as a response
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    }
    pub fn config(cfg: &mut web::ServiceConfig) {
        let client = Client::new();
        cfg.app_data(client)
            .route("/", web::get().to(Client::handle_index))
            .route("/submit", web::post().to(Client::handle_submit));
    }
}
pub async fn start_client() -> io::Result<()> {
    HttpServer::new(move || App::new().configure(Client::config))
        .bind("127.0.0.1:8000")
        .unwrap()
        .run()
        .await
}
