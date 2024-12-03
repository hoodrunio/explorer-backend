use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use regex::Regex;

use crate::{ProposalError, ProposalTypeInfo, ProposalField, ProposalTypeDiscovery};

pub struct ProtoDiscovery {
    proto_root: PathBuf,
    type_cache: tokio::sync::RwLock<HashMap<String, ProposalTypeInfo>>,
}

impl ProtoDiscovery {
    pub fn new<P: AsRef<Path>>(proto_root: P) -> Self {
        Self {
            proto_root: proto_root.as_ref().to_path_buf(),
            type_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    async fn scan_proto_files(&self) -> Result<Vec<PathBuf>, ProposalError> {
        let mut proto_files = Vec::new();
        
        for entry in WalkDir::new(&self.proto_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "proto") {
                proto_files.push(path.to_path_buf());
            }
        }
        
        Ok(proto_files)
    }

    async fn parse_proto_file(&self, path: &Path) -> Result<Vec<ProposalTypeInfo>, ProposalError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ProposalError::DiscoveryError(format!("Failed to read proto file: {}", e)))?;

        let mut types = Vec::new();
        let message_regex = Regex::new(r"message\s+(\w+)(?:Proposal)\s*\{([^}]+)\}")
            .map_err(|e| ProposalError::DiscoveryError(e.to_string()))?;
        
        let field_regex = Regex::new(r"(repeated\s+)?([.\w]+)\s+(\w+)\s*=\s*(\d+)")
            .map_err(|e| ProposalError::DiscoveryError(e.to_string()))?;

        for message_cap in message_regex.captures_iter(&content) {
            let type_name = message_cap.get(1).unwrap().as_str();
            let fields_str = message_cap.get(2).unwrap().as_str();
            
            let mut fields = Vec::new();
            for field_cap in field_regex.captures_iter(fields_str) {
                fields.push(ProposalField {
                    name: field_cap.get(3).unwrap().as_str().to_string(),
                    field_type: field_cap.get(2).unwrap().as_str().to_string(),
                    is_repeated: field_cap.get(1).is_some(),
                    number: field_cap.get(4).unwrap().as_str().parse().unwrap(),
                });
            }

            // Extract package name from proto file
            let package_regex = Regex::new(r"package\s+([\w.]+)")
                .map_err(|e| ProposalError::DiscoveryError(e.to_string()))?;
            
            let package = package_regex
                .captures(&content)
                .map(|cap| cap.get(1).unwrap().as_str())
                .unwrap_or("");

            let type_url = format!("/{}.{}Proposal", package, type_name);
            let relative_path = path.strip_prefix(&self.proto_root)
                .map_err(|e| ProposalError::DiscoveryError(e.to_string()))?;

            types.push(ProposalTypeInfo {
                type_url,
                proto_path: relative_path.to_string_lossy().into_owned(),
                fields,
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("package".to_string(), package.to_string());
                    map.insert("message_name".to_string(), type_name.to_string());
                    map
                },
            });
        }

        Ok(types)
    }
}

#[async_trait::async_trait]
impl ProposalTypeDiscovery for ProtoDiscovery {
    async fn discover_types(&self) -> Result<Vec<ProposalTypeInfo>, ProposalError> {
        let mut all_types = Vec::new();
        let proto_files = self.scan_proto_files().await?;
        
        for proto_file in proto_files {
            match self.parse_proto_file(&proto_file).await {
                Ok(mut types) => all_types.append(&mut types),
                Err(e) => eprintln!("Error parsing proto file {:?}: {}", proto_file, e),
            }
        }

        // Update cache
        let mut cache = self.type_cache.write().await;
        for type_info in &all_types {
            cache.insert(type_info.type_url.clone(), type_info.clone());
        }

        Ok(all_types)
    }

    async fn get_type_info(&self, type_url: &str) -> Result<ProposalTypeInfo, ProposalError> {
        // Try cache first
        {
            let cache = self.type_cache.read().await;
            if let Some(type_info) = cache.get(type_url) {
                return Ok(type_info.clone());
            }
        }

        // If not in cache, rediscover types
        self.discover_types().await?;
        
        // Try cache again
        let cache = self.type_cache.read().await;
        cache.get(type_url)
            .cloned()
            .ok_or_else(|| ProposalError::UnknownType(type_url.to_string()))
    }
}
