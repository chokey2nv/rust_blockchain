use std::error::Error;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
pub struct Article {
    pub author: String,
    pub timestamp: String,
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
            node: "".to_owned(),
        })
    }

    async fn handle_submit() -> impl Responder {
        // Extract form values from the PostObject
        HttpResponse::Ok().body("success")
    }
    // Handler function to render the index template
    async fn handle_index() -> impl Responder {
        let node_address = "http://example.com";
        let title = "My Blog";
        let articles = vec![
            Article {
                author: "John".to_string(),
                timestamp: "2023-07-20".to_string(),
                content: "Hello World!".to_string(),
            },
            Article {
                author: "Jane".to_string(),
                timestamp: "2023-07-21".to_string(),
                content: "Rust is awesome!".to_string(),
            },
        ];

        // Create an instance of the IndexTemplate with the data
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
    fn config(cfg: &mut web::ServiceConfig) {
        cfg.service(web::resource("/").route(web::get().to(Self::handle_index)))
            .service(web::resource("/submit").route(web::post().to(Self::handle_submit)));
    }
}
pub async fn start_client() {
    match HttpServer::new(move || App::new().configure(Client::config)).bind("127.0.0.1:8080") {
        Ok(server) => {
            println!("Server started at: http://127.0.0.1:8000");
            if let Err(e) = server.run().await {
                eprintln!("Error running server: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error binding to address: {}", e);
        }
    }
}
