use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;
use prost::Message;

mod discovery;
mod handler;
mod types;
mod descriptor;
#[cfg(test)]
mod tests;

pub use discovery::*;
pub use handler::*;
pub use types::*;
pub use descriptor::*;

#[derive(Debug, Error)]
pub enum ProposalError {
    #[error("Failed to parse proposal: {0}")]
    ParseError(String),
    #[error("Unknown proposal type: {0}")]
    UnknownType(String),
    #[error("Invalid proposal data: {0}")]
    InvalidData(String),
    #[error("Proto decode error: {0}")]
    ProtoDecodeError(String),
    #[error("Discovery error: {0}")]
    DiscoveryError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalInfo {
    pub title: String,
    pub description: String,
    pub type_url: String,
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[async_trait]
pub trait ProposalTypeDiscovery: Send + Sync {
    /// Discover proposal types from proto files
    async fn discover_types(&self) -> Result<Vec<ProposalTypeInfo>, ProposalError>;
    
    /// Get proposal type info by type URL
    async fn get_type_info(&self, type_url: &str) -> Result<ProposalTypeInfo, ProposalError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalTypeInfo {
    pub type_url: String,
    pub proto_path: String,
    pub fields: Vec<ProposalField>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalField {
    pub name: String,
    pub field_type: String,
    pub is_repeated: bool,
    pub number: i32,
}

#[async_trait]
pub trait ProposalHandler: Send + Sync {
    /// Handle a proposal of any type
    async fn handle(&self, type_info: &ProposalTypeInfo, data: &[u8]) -> Result<ProposalContent, ProposalError>;
    
    /// Returns true if this handler can handle the given type
    fn can_handle(&self, type_info: &ProposalTypeInfo) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalContent {
    pub title: String,
    pub description: String,
    pub type_url: String,
    pub content: serde_json::Value,
    pub metadata: HashMap<String, String>,
}
