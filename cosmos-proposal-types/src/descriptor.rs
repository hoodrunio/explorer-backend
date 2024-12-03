use std::path::{Path, PathBuf};
use prost_reflect::DescriptorPool;
use walkdir::WalkDir;
use dotenv::dotenv;
use std::env;

pub struct ProtoDescriptorGenerator {
    proto_root: PathBuf,
}

impl ProtoDescriptorGenerator {
    pub fn new<P: AsRef<Path>>(proto_root: P) -> Self {
        Self {
            proto_root: proto_root.as_ref().to_path_buf(),
        }
    }

    pub fn from_env() -> Self {
        dotenv().ok();
        Self::new(
            env::var("PROTO_ROOT_DIR")
                .unwrap_or_else(|_| "../protos".to_string())
        )
    }

    pub fn generate_file_descriptor_set(&self) -> prost_reflect::FileDescriptorSet {
        let mut file_descriptors = prost_reflect::FileDescriptorSet::default();
        let mut pool = DescriptorPool::new();

        // Collect all .proto files
        let mut proto_files = Vec::new();
        for entry in WalkDir::new(&self.proto_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().map_or(false, |ext| ext == "proto") {
                proto_files.push(entry.path().to_path_buf());
            }
        }

        // Sort proto files to ensure deterministic order
        proto_files.sort();

        // Process each proto file
        for proto_file in proto_files {
            if let Ok(descriptor) = self.compile_proto(&proto_file) {
                pool.register_file_descriptor_proto(&descriptor)
                    .expect("Failed to register file descriptor");
                file_descriptors.file.push(descriptor);
            }
        }

        file_descriptors
    }

    fn compile_proto(&self, proto_path: &Path) -> Result<prost_types::FileDescriptorProto, Box<dyn std::error::Error>> {
        let proto_dir = proto_path.parent().unwrap();
        let proto_file = proto_path.file_name().unwrap().to_str().unwrap();

        let mut config = protoc_rust::Codegen::new();
        config.out_dir(&proto_dir);
        config.include(&self.proto_root);
        config.input(&proto_file);
        config.generate_descriptors(true);

        let output = std::process::Command::new("protoc")
            .arg("--proto_path")
            .arg(&self.proto_root)
            .arg("--descriptor_set_out=/dev/stdout")
            .arg(proto_path)
            .output()?;

        if !output.status.success() {
            return Err(format!("protoc failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        let descriptor_set = prost_reflect::FileDescriptorSet::decode(&*output.stdout)?;
        Ok(descriptor_set.file.into_iter().next().unwrap())
    }

    pub fn get_descriptor_pool(&self) -> DescriptorPool {
        let file_descriptor_set = self.generate_file_descriptor_set();
        let mut pool = DescriptorPool::new();
        
        for file in file_descriptor_set.file {
            pool.register_file_descriptor_proto(&file)
                .expect("Failed to register file descriptor");
        }
        
        pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_generation() {
        let proto_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("protos");
            
        let generator = ProtoDescriptorGenerator::new(proto_root);
        let pool = generator.get_descriptor_pool();

        // Test that we can find common message types
        assert!(pool.get_message_by_name("cosmos.gov.v1beta1.TextProposal").is_some());
        assert!(pool.get_message_by_name("osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal").is_some());
        assert!(pool.get_message_by_name("lava.spec.SpecAddProposal").is_some());
    }
}
