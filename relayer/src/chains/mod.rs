pub mod ethereum;
pub mod solana;

use crate::events::{Chain, ChainEvent, Message};
use async_trait::async_trait;

#[async_trait]
pub trait ChainAdapter {
    async fn listen_events(&self);
    async fn submit_proof(&self, message: Message) -> Result<(), Box<dyn std::error::Error>>;
    fn get_chain_type(&self) -> Chain;
    async fn verify_proof(&self, message: &Message) -> bool;
}