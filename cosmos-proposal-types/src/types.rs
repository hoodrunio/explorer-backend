use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::{ProposalContent, ProposalHandler, ProposalError};

// Base proposal content implementation
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseProposal {
    pub title: String,
    pub description: String,
    pub proposal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

impl ProposalContent for BaseProposal {
    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn proposal_type(&self) -> &str {
        &self.proposal_type
    }

    fn to_json(&self) -> Result<String, ProposalError> {
        serde_json::to_string(self)
            .map_err(|e| ProposalError::ParseError(e.to_string()))
    }
}

// Example handler for text proposals
pub struct TextProposalHandler;

#[async_trait]
impl ProposalHandler for TextProposalHandler {
    async fn handle(&self, proposal_data: &[u8]) -> Result<Box<dyn ProposalContent>, ProposalError> {
        let proposal: BaseProposal = serde_json::from_slice(proposal_data)
            .map_err(|e| ProposalError::ParseError(e.to_string()))?;
        
        Ok(Box::new(proposal))
    }

    fn type_url(&self) -> &str {
        "/cosmos.gov.v1beta1.TextProposal"
    }
}

// Add more proposal types as needed
