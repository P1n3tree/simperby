use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use simperby_common::crypto::*;
use std::sync::Arc;
use tokio::sync::mpsc;

/// TODO: Provide error types.
///
/// Note: This trait is quite subject to change.
#[async_trait]
pub trait Network {
    /// Broadcasts a message to the network, after signed by the key given to this instance.
    async fn broadcast(&self, message: &[u8]) -> Result<(), String>;
    /// Creates a receiver for every message broadcasted to the network, except the one sent by this instance.
    async fn create_recv_queue(&self) -> Result<mpsc::Receiver<Vec<u8>>, ()>;
}

/// TODO: Provide error types.
///
/// Note: This trait is quite subject to change.
#[async_trait]
pub trait KVStore {
    /// Records the current state to the persistent storage.
    async fn commit_checkpoint(&mut self) -> Result<(), ()>;
    /// Reverts all the changes made since the last checkpoint.
    async fn revert_to_latest_checkpoint(&mut self) -> Result<(), ()>;
    /// Inserts a key-value pair into the store. If exists, it will be overwritten.
    async fn insert_or_update(&mut self, key: Hash256, value: &[u8]) -> Result<(), ()>;
    /// Removes a key-value pair from the store. If not exists, it will fail.
    async fn remove(&mut self, key: Hash256) -> Result<(), ()>;
    /// Retrieves the value associated with the key. If not exists, it will return `None`.
    async fn get(&self, key: Hash256) -> Result<Option<Vec<u8>>, ()>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Header {
    /// The author of this block.
    pub author: PublicKey,
    /// The signature of the previous block.
    pub prev_block_finalization_proof: Vec<Signature>,
    /// The hash of the previous block.
    pub previous_hash: Hash256,
    /// The height of this block.
    pub height: u64,
    /// The timestamp of this block.
    pub timestamp: u64,
    /// The Merkle root of transactions.
    pub tx_merkle_root: Hash256,
    /// The Merkle root of the state.
    pub state_merkle_root: Hash256,
    /// The hash of the set of validator & vote weight for the next block.
    pub validator_set_hash: Hash256,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Block<T> {
    pub header: Header,
    pub transactions: Vec<T>,
}

/// A state transition function.
#[async_trait]
pub trait BlockExecutor {
    type Transaction;
    async fn execute(
        &self,
        store: &mut dyn KVStore,
        transaction: Self::Transaction,
    ) -> Result<(), ()>;
}

/// A BFT consensus engine.
#[async_trait]
pub trait Consensus<E: BlockExecutor> {
    /// Peforms an one-block consensus.
    ///
    /// This method finishes when the next block is finalized.
    async fn progress(
        &mut self,
        block_to_propose: Option<Block<E::Transaction>>,
        last_finalized_header: Header,
        network: Arc<dyn Network>,
        executor: E,
        store: Box<dyn KVStore>,
    ) -> Result<Block<E::Transaction>, ()>;
}
