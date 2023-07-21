use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;

use super::chain::Transaction;

// Block represents a block in the blockchain.
#[derive(Debug, Serialize, Deserialize, Clone)] // Add Clone trait to the Block struct
pub struct Block {
    pub index: i32,
    pub transactions: Vec<Transaction>,
    pub timestamp: i64,
    pub previous_hash: String,
    pub nonce: i32,
    pub hash: String,
}

impl Block {
    // A function that returns the hash of the block contents.
    pub fn compute_hash(&self) -> Result<String, Box<dyn Error>> {
        // Use &self instead of &mut self
        let json_str = serde_json::to_string(self)?;
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    // Create a new block with a modified hash.
    pub fn with_modified_hash(&self, modified_hash: &str) -> Block {
        Block {
            index: self.index,
            transactions: self.transactions.clone(),
            timestamp: self.timestamp,
            previous_hash: self.previous_hash.clone(),
            nonce: self.nonce,
            hash: modified_hash.to_string(),
        }
    }
}
