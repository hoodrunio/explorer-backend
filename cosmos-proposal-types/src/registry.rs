use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{ProposalHandler, ProposalError};

#[derive(Default, Clone)]
pub struct ProposalRegistry {
    handlers: Arc<RwLock<HashMap<String, Box<dyn ProposalHandler + Send + Sync>>>>,
}

impl ProposalRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register<H>(&self, handler: H) -> Result<(), ProposalError>
    where
        H: ProposalHandler + Send + Sync + 'static,
    {
        let type_url = handler.type_url().to_string();
        let mut handlers = self.handlers.write().await;
        handlers.insert(type_url, Box::new(handler));
        Ok(())
    }

    pub async fn get_handler(&self, type_url: &str) -> Option<Box<dyn ProposalHandler + Send + Sync>> {
        let handlers = self.handlers.read().await;
        handlers.get(type_url).map(|h| h.clone())
    }

    pub async fn handle_proposal(&self, type_url: &str, data: &[u8]) -> Result<Box<dyn ProposalContent>, ProposalError> {
        let handler = self.get_handler(type_url).await
            .ok_or_else(|| ProposalError::UnknownType(type_url.to_string()))?;
        handler.handle(data).await
    }
}
