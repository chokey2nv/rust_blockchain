mod modules;

use crate::modules::app;

// In your main.rs, you can now start the server as follows:
#[tokio::main]
async fn main() {
    app::start_server().await
}
/*
#[tokio::main]
async fn main() {
    // Example usage of the blockchain in Rust:
    let mut blockchain = Blockchain::new_blockchain().unwrap();

    let transaction1 = Transaction {
        author: "John".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };
    blockchain.add_new_transaction(transaction1);

    let transaction2 = Transaction {
        author: "Alice".to_string(),
        content: "Hey there!".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };
    blockchain.add_new_transaction(transaction2);

    blockchain.mine_block().unwrap();

    // Call the announce_new_block() function using the await keyword
    match blockchain.announce_new_block().await {
        Ok(_) => println!("Block announcement successful!"),
        Err(e) => println!("Failed to announce block: {:?}", e),
    }

    // let result = blockchain.announce_new_block();

    println!("{:#?}", blockchain);
}
*/
