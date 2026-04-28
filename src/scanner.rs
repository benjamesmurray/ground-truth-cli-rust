use std::path::{Path, PathBuf};
use serde_json::Value as JsonValue;
use std::fs;

#[derive(Debug, Default, Clone)]
pub struct ProjectContext {
    pub language: Option<String>,
    pub framework: Option<String>,
    pub build_system: Option<String>,
    pub test_framework: Option<String>,
}

pub fn scan_project(path: &Path) -> ProjectContext {
    let mut context = ProjectContext::default();

    // Check for Rust
    if path.join("Cargo.toml").exists() {
        context.language = Some("rust".to_string());
        context.build_system = Some("cargo".to_string());
        context.test_framework = Some("cargo test".to_string());
        
        // Check for specific Rust frameworks/architectures
        if let Ok(cargo_toml) = fs::read_to_string(path.join("Cargo.toml")) {
            if cargo_toml.contains("tokio") {
                context.framework = Some("Asynchronous_Web_Microservices".to_string());
            } else if cargo_toml.contains("no_std") {
                context.framework = Some("Embedded_Bare_Metal_no_std".to_string());
            }
        }
    } 
    // Check for TypeScript/JavaScript
    else if path.join("package.json").exists() {
        context.language = Some("typescript".to_string());
        context.build_system = Some("npm/yarn".to_string());
        
        if let Ok(package_json_str) = fs::read_to_string(path.join("package.json")) {
            if let Ok(package_json) = serde_json::from_str::<JsonValue>(&package_json_str) {
                if let Some(deps) = package_json.get("dependencies").or(package_json.get("devDependencies")) {
                    if deps.get("next").is_some() {
                        context.framework = Some("NextJS_App_Router".to_string());
                    } else if deps.get("fastify").is_some() {
                        context.framework = Some("Fastify_High_Performance_API".to_string());
                    } else if deps.get("@aws-sdk/client-s3").is_some() {
                        context.framework = Some("AWS_Lambda_Serverless".to_string());
                    }
                }
                
                if let Some(scripts) = package_json.get("scripts") {
                    if scripts.get("test").is_some() {
                        context.test_framework = Some("npm test".to_string());
                    }
                }
            }
        }
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_rust_project() {
        let dir = tempdir().unwrap();
        let cargo_toml_path = dir.path().join("Cargo.toml");
        fs::write(cargo_toml_path, "tokio = \"1.0\"").unwrap();

        let context = scan_project(dir.path());
        assert_eq!(context.language, Some("rust".to_string()));
        assert_eq!(context.framework, Some("Asynchronous_Web_Microservices".to_string()));
    }

    #[test]
    fn test_scan_typescript_project() {
        let dir = tempdir().unwrap();
        let package_json_path = dir.path().join("package.json");
        fs::write(package_json_path, r#"{"dependencies": {"next": "14.0.0"}}"#).unwrap();

        let context = scan_project(dir.path());
        assert_eq!(context.language, Some("typescript".to_string()));
        assert_eq!(context.framework, Some("NextJS_App_Router".to_string()));
    }
}
