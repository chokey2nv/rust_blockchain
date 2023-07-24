use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::SystemTime;

use super::block::Block;

// Transaction represents a transaction in the blockchain.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub author: String,
    pub content: String,
    pub timestamp: i64,
}

// NodePeer represents a peer node in the blockchain network.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodePeer {
    pub node_address: String,
}

// Blockchain represents the blockchain and related operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub difficulty: i32,
    pub unconfirmed_transactions: Vec<Transaction>,
    pub chain: Vec<Block>,
    pub peers: Vec<NodePeer>,
}

impl Blockchain {
    // Create a new blockchain with a genesis block.
    pub fn new_blockchain() -> Result<Blockchain, Box<dyn Error>> {
        let mut bc = Blockchain {
            difficulty: 2,
            unconfirmed_transactions: Vec::new(),
            chain: Vec::new(),
            peers: Vec::new(),
        };
        bc.create_genesis_block()?;
        Ok(bc)
    }

    // Create a new blockchain by loading the blockchain data from a file.
    pub fn create_chain_from_file(dump: &str) -> Result<Blockchain, Box<dyn Error>> {
        let mut file = File::open(dump)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let blockchain: Blockchain = serde_json::from_str(&content)?;
        Ok(blockchain)
    }

    // Create a new blockchain by loading the blockchain data from a dump.
    pub fn create_chain_from_dump(
        chain_dump: Vec<serde_json::Map<String, serde_json::Value>>,
        node_addresses: Vec<String>,
    ) -> Result<Blockchain, Box<dyn Error>> {
        let mut generated_blockchain = Blockchain::new_blockchain()?;
        for (idx, block_data) in chain_dump.into_iter().enumerate() {
            if idx == 0 {
                continue; // Skip genesis block
            }

            let block = Block {
                index: block_data["index"].as_i64().unwrap() as i32,
                transactions: Blockchain::parse_transactions(
                    block_data["transactions"].as_array().unwrap(),
                )?,
                timestamp: block_data["timestamp"].as_i64().unwrap(),
                previous_hash: block_data["previous_hash"].as_str().unwrap().to_string(),
                nonce: block_data["nonce"].as_i64().unwrap() as i32,
                hash: block_data["hash"].as_str().unwrap().to_string(),
            };

            generated_blockchain.add_block(block)?;
        }
        let node_peers: Vec<NodePeer> = node_addresses
            .into_iter()
            .map(|node_address| NodePeer {
                node_address: node_address.to_string(),
            })
            .collect();
        generated_blockchain.peers = node_peers;
        Ok(generated_blockchain)
    }

    // Parse transactions converts transaction data from serde_json::Value to Vec<Transaction>.
    pub fn parse_transactions(
        transactions_data: &Vec<serde_json::Value>,
    ) -> Result<Vec<Transaction>, Box<dyn Error>> {
        let mut transactions = Vec::new();
        for transaction_data in transactions_data {
            let transaction = Transaction {
                author: transaction_data["author"].as_str().unwrap().to_string(),
                content: transaction_data["content"].as_str().unwrap().to_string(),
                timestamp: transaction_data["timestamp"].as_i64().unwrap(),
            };
            transactions.push(transaction);
        }
        Ok(transactions)
    }

    // Create the genesis block of the blockchain.
    pub fn create_genesis_block(&mut self) -> Result<(), Box<dyn Error>> {
        let genesis_block = Block {
            index: 0,
            transactions: Vec::new(),
            timestamp: 0,
            previous_hash: "0".to_string(),
            nonce: 0,
            hash: "".to_string(),
        };
        let computed_hash = genesis_block.compute_hash()?;
        self.chain.push(Block {
            hash: computed_hash,
            ..genesis_block
        });
        Ok(())
    }

    // Get the last block in the chain.
    pub fn get_last_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    // Add the block to the chain after verification.
    pub fn add_block(&mut self, block: Block) -> Result<(), Box<dyn Error>> {
        // Compare the previous hash.
        if self.get_last_block().hash != block.previous_hash {
            return Err("Previous hash incorrect".into());
        }

        if !self.is_valid_proof(&block, &block.hash) {
            return Err("Block proof invalid".into());
        }

        self.chain.push(block);
        Ok(())
    }

    // Add the pending transactions to the blockchain by adding them to a block and figuring out Proof of Work.
    pub fn mine_block(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.unconfirmed_transactions.is_empty() {
            return Ok(false);
        }

        let last_block = self.get_last_block();
        let index = last_block.index + 1;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;
        let previous_hash = last_block.hash.clone();

        let mut new_block = Block {
            index,
            transactions: self.unconfirmed_transactions.clone(),
            timestamp,
            previous_hash,
            nonce: 0,
            hash: "".to_string(),
        };

        self.proof_of_work(&mut new_block)?;
        self.add_block(new_block)?;

        self.unconfirmed_transactions.clear();
        Ok(true)
    }

    // Add a new node peer to the blockchain.
    pub fn add_node_peer(&mut self, node: NodePeer) {
        self.peers.push(node);
    }

    // Add a new transaction to the list of unconfirmed transactions.
    pub fn add_new_transaction(&mut self, transaction: Transaction) {
        self.unconfirmed_transactions.push(transaction);
    }

    // Perform the Proof of Work algorithm to find a hash that satisfies the difficulty criteria.
    pub fn proof_of_work(&self, block: &mut Block) -> Result<(), Box<dyn Error>> {
        let prefix = "0".repeat(self.difficulty as usize);
        while !block.hash.starts_with(&prefix) {
            block.nonce += 1;
            let hash = block.compute_hash()?;
            block.hash = hash;
        }
        Ok(())
    }

    // Announce a new block to the network.
    pub async fn announce_new_block(&self) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();
        for peer in &self.peers {
            let url = format!("{}/add_block", peer.node_address);

            let json_str = serde_json::to_string(&self.get_last_block())?;
            let response = client.post(&url).body(json_str).send().await?;

            if response.status().is_success() {
                println!("Block added to node {}", peer.node_address);
            } else {
                println!(
                    "Failed to add block to node {}: {}",
                    peer.node_address,
                    response.status()
                );
            }
        }
        Ok(())
    }

    // Perform consensus - If a longer valid chain is found, our chain is replaced with it.
    pub async fn consensus(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut current_len = self.chain.len() as i64;
        let mut longest_chain = Vec::new();
        for peer in &self.peers {
            let res_body = reqwest::get(&format!("{}/chain", peer.node_address))
                .await?
                .bytes()
                .await?;

            let json_data: serde_json::Value = serde_json::from_slice(&res_body)?;
            let length = json_data["length"].as_i64().unwrap();
            let chain_data: Vec<serde_json::Map<String, serde_json::Value>> = json_data["chain"]
                .as_array()
                .ok_or("Invalid JSON format")?
                .iter()
                .map(|item| {
                    if let Some(obj) = item.as_object() {
                        obj.clone()
                    } else {
                        serde_json::Map::new()
                    }
                })
                .collect();
            let new_blockchain =
                Blockchain::create_chain_from_dump(chain_data.to_vec(), vec![]).unwrap();

            if length > current_len && new_blockchain.check_chain_validity() {
                current_len = length;
                longest_chain = chain_data.to_vec();
            }
        }

        if !longest_chain.is_empty() {
            self.chain = Blockchain::create_chain_from_dump(longest_chain, vec![])
                .unwrap()
                .chain;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Check if the given block hash is a valid proof of work and satisfies the difficulty criteria.
    pub fn is_valid_proof(&self, block: &Block, block_hash: &str) -> bool {
        let copy_block = block.with_modified_hash(""); // Create a new block with an empty hash
        let computed_hash = copy_block.compute_hash().unwrap();
        block_hash.starts_with(&"0".repeat(self.difficulty as usize)) && block_hash == computed_hash
    }

    // Check the validity of the blockchain by verifying each block and its hash.
    pub fn check_chain_validity(&self) -> bool {
        let mut previous_hash = "0".to_string();

        for block in &self.chain {
            if block.index != 0
                && (!self.is_valid_proof(block, &block.hash)
                    || previous_hash != block.previous_hash)
            {
                return false;
            }
            previous_hash = block.hash.clone();
        }

        true
    }
}
