use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use dotenv::dotenv;
use std::env;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::time::Instant;

#[derive(Debug)]
struct ProtoFile {
    path: PathBuf,
    imports: Vec<String>,
    package: String,
    messages: HashSet<String>,
}

impl ProtoFile {
    fn new(path: PathBuf, content: &str) -> Self {
        let mut imports = Vec::new();
        let mut package = String::new();
        let mut messages = HashSet::new();
        let mut in_message = false;
        let mut current_message = String::new();
        let mut message_types = HashSet::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("import") {
                if let Some(import) = line.split('"').nth(1) {
                    imports.push(import.to_string());
                }
            } else if line.starts_with("package") {
                if let Some(pkg) = line.strip_prefix("package").and_then(|s| s.trim().strip_suffix(";")) {
                    package = pkg.trim().to_string();
                }
            } else if line.starts_with("message") {
                in_message = true;
                if let Some(name) = line.split_whitespace().nth(1) {
                    current_message = name.to_string();
                }
            } else if in_message {
                if line.contains("}") {
                    in_message = false;
                    if !current_message.is_empty() {
                        messages.insert(current_message.clone());
                    }
                } else {
                    // Extract message types used within messages
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        message_types.insert(parts[0].to_string());
                    }
                }
            }
        }

        // Add implicit imports for related proto files
        if !package.is_empty() {
            let package_parts: Vec<_> = package.split('.').collect();
            if package_parts.len() > 1 {
                let base_path = package_parts[..package_parts.len()-1].join("/");
                
                // Add imports for related proto files in the same package
                if let Some(parent) = path.parent() {
                    for entry in std::fs::read_dir(parent).unwrap_or_else(|_| panic!("Failed to read dir")) {
                        if let Ok(entry) = entry {
                            let file_name = entry.file_name();
                            let file_name_str = file_name.to_string_lossy();
                            if file_name_str.ends_with(".proto") && !imports.iter().any(|i| i.ends_with(&file_name_str.to_string())) {
                                let import_path = format!("{}/{}", base_path, file_name_str);
                                imports.push(import_path);
                            }
                        }
                    }
                }
            }
        }
        
        ProtoFile { path, imports, package, messages }
    }

    fn is_likely_duplicate(&self, other: &ProtoFile) -> bool {
        if self.package != other.package {
            return false;
        }

        // For legacy files, always consider them duplicates of their main version
        if self.path.to_string_lossy().contains("_legacy") || 
           other.path.to_string_lossy().contains("_legacy") {
            return true;
        }

        // Check message overlap - if they define the same messages, they're likely duplicates
        let message_overlap: HashSet<_> = self.messages.intersection(&other.messages).collect();
        !message_overlap.is_empty()
    }

    fn calculate_priority(&self) -> usize {
        let mut score = self.path.components().count();
        
        // Penalize files with more imports
        score += self.imports.len();

        // Penalize legacy files
        if self.path.to_string_lossy().contains("_legacy") {
            score += 1000; // Large penalty to ensure legacy files are never chosen over main files
        }

        score
    }
}

struct ProtoDependencyResolver {
    files: HashMap<String, ProtoFile>,
    well_known_prefixes: HashSet<&'static str>,
    package_paths: HashMap<String, PathBuf>,
}

impl ProtoDependencyResolver {
    fn new() -> Self {
        let well_known_prefixes: HashSet<&'static str> = [
            "google/",
            "cosmos/",
            "gogoproto/",
            "cosmos_proto/",
            "tendermint/",
            "ibc/",
        ].iter().copied().collect();

        Self {
            files: HashMap::new(),
            well_known_prefixes,
            package_paths: HashMap::new(),
        }
    }

    fn add_file(&mut self, path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        let new_proto = ProtoFile::new(path.to_path_buf(), &content);
        
        if new_proto.package.is_empty() {
            println!("Warning: Skipping {} due to missing package definition", path.display());
            return Ok(());
        }

        // Find any existing files with the same package
        let mut duplicates: Vec<_> = self.files.values()
            .filter(|existing| existing.is_likely_duplicate(&new_proto))
            .collect();

        if !duplicates.is_empty() {
            duplicates.sort_by_key(|p| p.calculate_priority());
            let best_priority = duplicates[0].calculate_priority();
            let new_priority = new_proto.calculate_priority();

            if new_priority > best_priority {
                println!("Warning: Skipping lower priority proto at {} (priority {})", 
                    path.display(), new_priority);
                return Ok(());
            } else {
                // Remove existing lower priority duplicates
                self.files.retain(|_, proto| {
                    !proto.is_likely_duplicate(&new_proto) || 
                    proto.calculate_priority() <= new_priority
                });
                println!("Note: Using higher priority proto at {} (priority {})", 
                    path.display(), new_priority);
            }
        }

        let path_str = path.to_string_lossy().into_owned();
        self.package_paths.insert(new_proto.package.clone(), path.to_path_buf());
        self.files.insert(path_str, new_proto);
        Ok(())
    }

    fn is_well_known(&self, import: &str) -> bool {
        self.well_known_prefixes.iter().any(|prefix| import.starts_with(prefix))
    }

    fn get_required_imports(&self, file: &ProtoFile) -> HashSet<String> {
        file.imports.iter()
            .filter(|import| !self.is_well_known(import))
            .cloned()
            .collect()
    }

    fn resolve_dependencies(&self) -> Vec<PathBuf> {
        let start = Instant::now();
        println!("Starting dependency resolution...");
        
        let mut resolved = HashSet::new();
        let mut result = Vec::new();

        for (path, file) in &self.files {
            let required_imports = self.get_required_imports(file);
            if required_imports.is_empty() {
                resolved.insert(path.clone());
                result.push(file.path.clone());
            }
        }

        println!("Found {} files with only well-known imports", resolved.len());

        let mut last_resolved_count = 0;
        while resolved.len() > last_resolved_count {
            last_resolved_count = resolved.len();

            for (path, file) in &self.files {
                if resolved.contains(path) {
                    continue;
                }

                let required_imports = self.get_required_imports(file);
                let all_deps_resolved = required_imports
                    .iter()
                    .all(|import| {
                        self.is_well_known(import) || 
                        resolved.iter().any(|r| r.contains(import))
                    });

                if all_deps_resolved {
                    resolved.insert(path.clone());
                    result.push(file.path.clone());
                }
            }
        }

        println!("Dependency resolution completed in {:?}", start.elapsed());
        println!("Total files: {}, Resolved: {}", self.files.len(), result.len());

        result.sort();
        result
    }
}

fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n")
}

fn fix_proto_syntax(content: &str) -> String {
    let content = normalize_line_endings(content);
    let mut fixed_lines = Vec::new();
    let mut in_message = false;
    let mut field_number = 1;
    let mut brace_count = 0;

    // First pass: fix line continuations and multiline strings
    let mut processed_lines = Vec::new();
    let mut current_line = String::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current_line.is_empty() {
                processed_lines.push(current_line);
                current_line = String::new();
            }
            continue;
        }

        // Handle line continuations and merge multiline strings
        if current_line.ends_with('\\') || 
           (current_line.contains("\"") && !current_line.matches("\"").count() % 2 == 0) {
            current_line.push_str(trimmed);
        } else if !current_line.is_empty() {
            processed_lines.push(current_line);
            current_line = String::from(trimmed);
        } else {
            current_line = String::from(trimmed);
        }
    }
    if !current_line.is_empty() {
        processed_lines.push(current_line);
    }

    // Second pass: fix syntax
    // Always start with syntax and package declarations
    fixed_lines.push(String::from("syntax = \"proto3\";"));

    let mut has_package = false;
    let mut imports = Vec::new();
    let mut messages = Vec::new();
    let mut services = Vec::new();
    let mut options = Vec::new();
    let mut others = Vec::new();

    // Collect and categorize statements
    let mut i = 0;
    while i < processed_lines.len() {
        let line = &processed_lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("syntax") {
            i += 1;
            continue;
        }

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            i += 1;
            continue;
        }

        if trimmed.starts_with("package") {
            if !has_package {
                let mut pkg_line = trimmed.to_string();
                if !pkg_line.ends_with(';') {
                    pkg_line.push(';');
                }
                has_package = true;
                fixed_lines.push(pkg_line);
            }
            i += 1;
        } else if trimmed.starts_with("import") {
            if let Some(start_quote) = trimmed.find('"') {
                if let Some(end_quote) = trimmed[start_quote + 1..].find('"') {
                    let import_path = &trimmed[start_quote..=start_quote + end_quote + 1];
                    imports.push(format!("import {};", import_path));
                }
            }
            i += 1;
        } else if trimmed.starts_with("message") {
            let mut message_lines = Vec::new();
            let mut message_brace_count = 0;
            let mut current_field_number = 1;

            // Extract message name
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                let name = name.trim_matches('{').trim();
                message_lines.push(format!("message {} {{", name));
                message_brace_count += 1;

                // Process message body
                i += 1;
                while i < processed_lines.len() && message_brace_count > 0 {
                    let body_line = &processed_lines[i];
                    let body_trimmed = body_line.trim();
                    
                    if body_trimmed.contains('{') {
                        message_brace_count += 1;
                    }
                    if body_trimmed.contains('}') {
                        message_brace_count -= 1;
                    }

                    if message_brace_count == 0 {
                        message_lines.push(String::from("}"));
                    } else if body_trimmed.starts_with("//") || body_trimmed.starts_with("/*") || body_trimmed.starts_with("option") {
                        message_lines.push(String::from(body_trimmed));
                    } else {
                        let mut parts: Vec<&str> = body_trimmed.split_whitespace().collect();
                        if !parts.is_empty() {
                            let mut field_line = String::new();

                            // Add field modifier
                            if !["required", "optional", "repeated"].contains(&parts[0]) {
                                field_line.push_str("optional ");
                            } else {
                                field_line.push_str(parts[0]);
                                field_line.push_str(" ");
                                parts.remove(0);
                            }

                            if !parts.is_empty() {
                                // Add type and field name
                                field_line.push_str(&parts[0]); // type
                                if parts.len() > 1 {
                                    field_line.push_str(" ");
                                    field_line.push_str(&parts[1].trim_matches(';')); // name
                                }

                                // Add field number
                                field_line.push_str(&format!(" = {}", current_field_number));
                                current_field_number += 1;

                                // Add semicolon
                                if !field_line.ends_with(';') {
                                    field_line.push(';');
                                }

                                message_lines.push(format!("    {}", field_line));
                            }
                        }
                    }
                    i += 1;
                }
            }
            messages.push(message_lines);
        } else if trimmed.starts_with("service") {
            services.push(trimmed.to_string());
            i += 1;
        } else if trimmed.starts_with("option") {
            let mut option_line = trimmed.to_string();
            if !option_line.ends_with(';') {
                option_line.push(';');
            }
            options.push(option_line);
            i += 1;
        } else if !trimmed.is_empty() {
            let mut other_line = trimmed.to_string();
            if !other_line.ends_with(';') && !other_line.ends_with('{') && !other_line.ends_with('}') {
                other_line.push(';');
            }
            others.push(other_line);
            i += 1;
        } else {
            i += 1;
        }
    }

    // Add imports
    fixed_lines.extend(imports);

    // Add options
    fixed_lines.extend(options);

    // Add other top-level statements
    fixed_lines.extend(others);

    // Add messages
    for message in messages {
        fixed_lines.extend(message);
    }

    // Add services
    fixed_lines.extend(services);

    fixed_lines.join("\n")
}

fn process_proto_file(proto_path: &Path, _proto_root: &Path) -> std::io::Result<bool> {
    let content = fs::read_to_string(proto_path)?;
    
    // Fix proto syntax and create backup
    let fixed_content = fix_proto_syntax(&content);
    if fixed_content != content {
        let backup_path = proto_path.with_extension("proto.bak");
        fs::write(&backup_path, &content)?;
        println!("Created backup at {}", backup_path.display());
        fs::write(proto_path, fixed_content)?;
        println!("Fixed syntax in {}", proto_path.display());
    }

    // Process imports
    let content = fs::read_to_string(proto_path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    let mut imports = HashSet::new();
    let mut needs_update = false;

    // Extract package name and collect imports
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("import") {
            if let Some(import) = line.split('"').nth(1) {
                imports.insert(import.to_string());
            }
        }
    }

    // Find message references that need imports
    let mut missing_imports = HashSet::new();
    for line in &lines {
        let trimmed = line.trim();
        if !trimmed.starts_with("import") && !trimmed.starts_with("package") && !trimmed.starts_with("syntax") {
            for word in line.split_whitespace() {
                if word.contains('.') && !word.starts_with("google.protobuf") {
                    let parts: Vec<_> = word.split('.').collect();
                    if parts.len() > 1 {
                        let import_path = format!("{}.proto", parts[..parts.len()-1].join("/"));
                        if !imports.contains(&import_path) {
                            missing_imports.insert(import_path);
                            needs_update = true;
                        }
                    }
                }
            }
        }
    }

    // Add missing imports if needed
    if needs_update {
        let mut new_lines = Vec::new();
        let mut added_imports = false;

        for line in lines {
            if !added_imports && line.trim().starts_with("package") {
                new_lines.push(line);
                for import in &missing_imports {
                    new_lines.push(format!("import \"{}\";", import));
                }
                added_imports = true;
            } else {
                new_lines.push(line);
            }
        }

        fs::write(proto_path, new_lines.join("\n"))?;
        println!("Updated imports in {}", proto_path.display());
    }

    Ok(true)
}

fn is_proto3_syntax(content: &str) -> bool {
    content.lines()
        .map(|line| line.trim())
        .any(|line| line == "syntax = \"proto3\";")
}

fn preprocess_proto_files(protos: &[PathBuf], proto_root: &Path) -> Vec<PathBuf> {
    let mut valid_protos = Vec::new();
    
    for proto in protos {
        match process_proto_file(proto, proto_root) {
            Ok(true) => valid_protos.push(proto.clone()),
            Ok(false) => println!("Skipping incompatible proto file: {}", proto.display()),
            Err(e) => println!("Warning: Failed to process {}: {}", proto.display(), e),
        }
    }
    
    valid_protos
}

fn main() {
    let start = Instant::now();
    dotenv().ok();
    
    let proto_root = {
        let workspace_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        workspace_dir.join(
            env::var("PROTO_ROOT_DIR")
                .unwrap_or_else(|_| "protos".to_string())
        )
    };
    
    println!("Proto directory: {}", proto_root.display());

    // Create common proto directories if they don't exist
    let common_protos = proto_root.join("google").join("api");
    fs::create_dir_all(&common_protos).unwrap_or_else(|_| {
        println!("Warning: Could not create common proto directory: {}", common_protos.display());
    });

    // Write common proto files
    let http_proto = common_protos.join("http.proto");
    if !http_proto.exists() {
        fs::write(&http_proto, r#"syntax = "proto3";

package google.api;

message Http {
  repeated HttpRule rules = 1;
}

message HttpRule {
  oneof pattern {
    string get = 2;
    string put = 3;
    string post = 4;
    string delete = 5;
    string patch = 6;
    CustomPattern custom = 8;
  }
  string selector = 1;
  string body = 7;
  repeated HttpRule additional_bindings = 11;
}

message CustomPattern {
  string kind = 1;
  string path = 2;
}"#).unwrap_or_else(|_| {
        println!("Warning: Could not write http.proto");
    });
    }

    let service_proto = proto_root.join("service.proto");
    if !service_proto.exists() {
        fs::write(&service_proto, r#"syntax = "proto3";

package service;

import "google/api/http.proto";

extend google.protobuf.MethodOptions {
  HttpRule http = 72295728;
}"#).unwrap_or_else(|_| {
        println!("Warning: Could not write service.proto");
    });
    }

    let mut resolver = ProtoDependencyResolver::new();

    let mut file_count = 0;
    let mut protos = Vec::new();
    for entry in WalkDir::new(&proto_root).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |ext| ext == "proto") {
            protos.push(entry.path().to_path_buf());
            if let Err(e) = resolver.add_file(entry.path()) {
                println!("Warning: Failed to analyze {}: {}", entry.path().display(), e);
            }
            file_count += 1;
        }
    }
    println!("Found {} proto files in {:?}", file_count, start.elapsed());

    // Preprocess and filter proto files
    println!("Processing {} proto files...", protos.len());
    let valid_protos = preprocess_proto_files(&protos, &proto_root);
    println!("Found {} valid proto3 files", valid_protos.len());

    if valid_protos.is_empty() {
        println!("No valid proto3 files found in {}", proto_root.display());
        return;
    }

    let protoc_args = vec![
        "--experimental_allow_proto3_optional".to_string(),
    ];

    // Configure tonic-build with the processed proto files
    let mut config = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional");

    // Add proto root to include path
    config = config.include_file("mod.rs")
        .compile_well_known_types(true)
        .protoc_arg("-I")
        .protoc_arg(proto_root.to_str().unwrap());

    // Add common proto directories to include path
    let common_proto_dirs = [
        proto_root.join("google").join("api"),
        proto_root.clone(),
    ];

    for dir in &common_proto_dirs {
        config = config.protoc_arg("-I").protoc_arg(dir.to_str().unwrap());
    }

    // Compile the proto files
    match config.compile(&valid_protos, &[proto_root.clone()]) {
        Ok(_) => println!("Successfully compiled {} proto files", valid_protos.len()),
        Err(e) => panic!("Failed to compile protos: {}", e),
    }

    println!("Build completed in {:?}", start.elapsed());

    println!("cargo:rerun-if-changed=.env");
    for proto in valid_protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }
}
