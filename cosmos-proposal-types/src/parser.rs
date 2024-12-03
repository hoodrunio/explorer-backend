use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use crate::{ProposalError, ProposalInfo, ProposalRegistry};

pub struct ProposalParser {
    registry: ProposalRegistry,
}

impl ProposalParser {
    pub fn new(registry: ProposalRegistry) -> Self {
        Self { registry }
    }

    pub async fn parse_proposal(&self, proposal_info: ProposalInfo) -> Result<Box<dyn ProposalContent>, ProposalError> {
        let content_data = BASE64.decode(proposal_info.content)
            .map_err(|e| ProposalError::ParseError(format!("Failed to decode base64 content: {}", e)))?;

        self.registry.handle_proposal(&proposal_info.type_url, &content_data).await
    }
}
