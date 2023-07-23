mod modules;

use crate::modules::{app::start_node, client::start_client};

// In your main.rs, you can now start the server as follows:
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Start both functions concurrently using tokio::spawn
    start_client();
    start_node();

    // Wait for both servers to finish (not necessary in this example)
    // You can remove this line if you don't want to wait for the servers to finish.
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}
