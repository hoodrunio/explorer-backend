#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use pretty_assertions::assert_eq;
    use prost::Message;
    use prost_reflect::{DynamicMessage, Value};

    // Helper function to get descriptor pool
    fn get_descriptor_pool() -> prost_reflect::DescriptorPool {
        let generator = ProtoDescriptorGenerator::from_env();
        generator.get_descriptor_pool()
    }

    #[tokio::test]
    async fn test_proto_discovery() {
        let discovery = ProtoDiscovery::new(ProtoDescriptorGenerator::from_env().proto_root);
        let types = discovery.discover_types().await.unwrap();
        
        // Verify we found some proposal types
        assert!(!types.is_empty());
        
        // Verify we found common proposal types
        let type_urls: Vec<&str> = types.iter()
            .map(|t| t.type_url.as_str())
            .collect();

        // Cosmos gov proposals
        assert!(type_urls.contains("/cosmos.gov.v1beta1.TextProposal"));
        
        // Osmosis proposals
        assert!(type_urls.contains("/osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal"));
        assert!(type_urls.contains("/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal"));
        
        // Lava proposals
        assert!(type_urls.contains("/lava.spec.SpecAddProposal"));
        assert!(type_urls.contains("/lava.plans.PlansAddProposal"));
    }

    #[tokio::test]
    async fn test_cosmos_text_proposal() {
        let discovery = ProtoDiscovery::new(ProtoDescriptorGenerator::from_env().proto_root);
        let type_info = discovery.get_type_info("/cosmos.gov.v1beta1.TextProposal").await.unwrap();
        
        // Verify fields
        assert_eq!(type_info.fields.len(), 2);
        assert!(type_info.fields.iter().any(|f| f.name == "title"));
        assert!(type_info.fields.iter().any(|f| f.name == "description"));
        
        // Create a test proposal using dynamic message
        let pool = get_descriptor_pool();
        let message_desc = pool.get_message_by_name("cosmos.gov.v1beta1.TextProposal").unwrap();
        let mut dynamic_msg = DynamicMessage::new(message_desc);
        
        dynamic_msg.set_field_by_name("title", Value::String("Test Proposal".to_string()));
        dynamic_msg.set_field_by_name("description", Value::String("This is a test proposal".to_string()));
        
        let mut handler = DynamicProposalHandler::new();
        handler.register_descriptor_pool(pool);
        
        // Encode proposal
        let data = dynamic_msg.encode_to_vec();
        
        // Handle proposal
        let content = handler.handle(&type_info, &data).await.unwrap();
        
        assert_eq!(content.title, "Test Proposal");
        assert_eq!(content.description, "This is a test proposal");
        assert_eq!(content.type_url, "/cosmos.gov.v1beta1.TextProposal");
    }

    #[tokio::test]
    async fn test_osmosis_pool_incentives_proposal() {
        let discovery = ProtoDiscovery::new(ProtoDescriptorGenerator::from_env().proto_root);
        let type_info = discovery
            .get_type_info("/osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal")
            .await
            .unwrap();
        
        // Verify fields
        assert!(type_info.fields.iter().any(|f| f.name == "title"));
        assert!(type_info.fields.iter().any(|f| f.name == "description"));
        assert!(type_info.fields.iter().any(|f| f.name == "records"));
        
        // Create a test proposal using dynamic message
        let pool = get_descriptor_pool();
        let message_desc = pool.get_message_by_name("osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal").unwrap();
        let mut dynamic_msg = DynamicMessage::new(message_desc);
        
        // Create a DistrRecord message
        let distr_record_desc = pool.get_message_by_name("osmosis.poolincentives.v1beta1.DistrRecord").unwrap();
        let mut distr_record = DynamicMessage::new(distr_record_desc);
        distr_record.set_field_by_name("gauge_id", Value::U64(1));
        distr_record.set_field_by_name("weight", Value::String("10".to_string()));
        
        dynamic_msg.set_field_by_name("title", Value::String("Update Pool Incentives".to_string()));
        dynamic_msg.set_field_by_name("description", Value::String("Updating incentives distribution".to_string()));
        dynamic_msg.set_field_by_name("records", Value::List(vec![Value::Message(distr_record)]));
        
        let mut handler = DynamicProposalHandler::new();
        handler.register_descriptor_pool(pool);
        
        // Encode proposal
        let data = dynamic_msg.encode_to_vec();
        
        // Handle proposal
        let content = handler.handle(&type_info, &data).await.unwrap();
        
        assert_eq!(content.title, "Update Pool Incentives");
        assert_eq!(content.description, "Updating incentives distribution");
        
        // Verify records in content
        let records = content.content.get("records").unwrap().as_array().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get("gauge_id").unwrap().as_u64().unwrap(), 1);
        assert_eq!(records[0].get("weight").unwrap().as_str().unwrap(), "10");
    }

    #[tokio::test]
    async fn test_lava_spec_proposal() {
        let discovery = ProtoDiscovery::new(ProtoDescriptorGenerator::from_env().proto_root);
        let type_info = discovery
            .get_type_info("/lava.spec.SpecAddProposal")
            .await
            .unwrap();
        
        // Verify fields
        assert!(type_info.fields.iter().any(|f| f.name == "title"));
        assert!(type_info.fields.iter().any(|f| f.name == "description"));
        
        // Create a test proposal using dynamic message
        let pool = get_descriptor_pool();
        let message_desc = pool.get_message_by_name("lava.spec.SpecAddProposal").unwrap();
        let mut dynamic_msg = DynamicMessage::new(message_desc);
        
        dynamic_msg.set_field_by_name("title", Value::String("Add New Spec".to_string()));
        dynamic_msg.set_field_by_name("description", Value::String("Adding a new specification".to_string()));
        
        let mut handler = DynamicProposalHandler::new();
        handler.register_descriptor_pool(pool);
        
        // Encode proposal
        let data = dynamic_msg.encode_to_vec();
        
        // Handle proposal
        let content = handler.handle(&type_info, &data).await.unwrap();
        
        assert_eq!(content.title, "Add New Spec");
        assert_eq!(content.description, "Adding a new specification");
    }
}
