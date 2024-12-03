use std::collections::HashMap;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, Value};
use crate::{ProposalError, ProposalTypeInfo, ProposalHandler, ProposalContent};

pub struct DynamicProposalHandler {
    descriptor_pool: Option<DescriptorPool>,
}

impl DynamicProposalHandler {
    pub fn new() -> Self {
        Self {
            descriptor_pool: None,
        }
    }

    pub fn register_descriptor_pool(&mut self, pool: DescriptorPool) {
        self.descriptor_pool = Some(pool);
    }

    fn decode_dynamic_message(&self, type_url: &str, data: &[u8]) -> Result<DynamicMessage, ProposalError> {
        let pool = self.descriptor_pool.as_ref()
            .ok_or_else(|| ProposalError::ProtoDecodeError("No descriptor pool registered".to_string()))?;

        let message_desc = pool.get_message_by_name(type_url)
            .ok_or_else(|| ProposalError::UnknownType(type_url.to_string()))?;

        DynamicMessage::decode(message_desc, data)
            .map_err(|e| ProposalError::ProtoDecodeError(e.to_string()))
    }

    fn extract_field_value(value: &Value) -> serde_json::Value {
        match value {
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::I32(i) => serde_json::Value::Number((*i).into()),
            Value::I64(i) => serde_json::Value::Number((*i).into()),
            Value::U32(u) => serde_json::Value::Number((*u).into()),
            Value::U64(u) => serde_json::Value::Number((*u).into()),
            Value::F32(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f as f64).unwrap_or_default()),
            Value::F64(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or_default()),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Bytes(b) => serde_json::Value::String(base64::encode(b)),
            Value::Message(m) => {
                let mut map = serde_json::Map::new();
                for field in m.descriptor().fields() {
                    if let Some(val) = m.get_field(&field) {
                        map.insert(
                            field.name().to_string(),
                            Self::extract_field_value(val),
                        );
                    }
                }
                serde_json::Value::Object(map)
            }
            Value::List(l) => {
                serde_json::Value::Array(
                    l.iter()
                        .map(Self::extract_field_value)
                        .collect(),
                )
            }
            Value::Enum(e) => {
                serde_json::Value::String(e.name().to_string())
            }
            _ => serde_json::Value::Null,
        }
    }
}

#[async_trait::async_trait]
impl ProposalHandler for DynamicProposalHandler {
    async fn handle(&self, type_info: &ProposalTypeInfo, data: &[u8]) -> Result<ProposalContent, ProposalError> {
        let dynamic_message = self.decode_dynamic_message(&type_info.type_url, data)?;
        
        let mut content_map = serde_json::Map::new();
        for field in dynamic_message.descriptor().fields() {
            if let Some(value) = dynamic_message.get_field(&field) {
                content_map.insert(
                    field.name().to_string(),
                    Self::extract_field_value(value),
                );
            }
        }

        // Extract title and description from the message
        let title = content_map.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let description = content_map.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(ProposalContent {
            title,
            description,
            type_url: type_info.type_url.clone(),
            content: serde_json::Value::Object(content_map),
            metadata: type_info.metadata.clone(),
        })
    }

    fn can_handle(&self, type_info: &ProposalTypeInfo) -> bool {
        if let Some(pool) = &self.descriptor_pool {
            pool.get_message_by_name(&type_info.type_url).is_some()
        } else {
            false
        }
    }
}
