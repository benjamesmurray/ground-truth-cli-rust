use rust_embed::RustEmbed;
use crate::scanner::ProjectContext;
use std::fs;
use std::path::Path;
use serde_json::Value as JsonValue;

#[derive(RustEmbed)]
#[folder = "rules/"]
struct Asset;

pub fn synthesize_rules(
    context: &ProjectContext, 
    output_path: &Path, 
    intent: Option<&str>, 
    include_guidance: bool
) -> anyhow::Result<()> {
    let mut content = String::new();

    // 1. Load ground_truth_rules.toon
    if let Some(core_rules) = Asset::get("ground_truth_rules.toon") {
        let core_str = std::str::from_utf8(core_rules.data.as_ref())?;
        
        // Split core rules into preamble and guidance
        let parts: Vec<&str> = core_str.split("expert_dev_guidance:").collect();
        let mut zone1 = parts[0].to_string();
        
        // Replace placeholders in Zone 1
        zone1 = zone1.replace("[CONTEXT_LIMIT]", "200,000 tokens");
        zone1 = zone1.replace("[BUILD_SYSTEM]", context.build_system.as_deref().unwrap_or("Unknown"));
        zone1 = zone1.replace("[TEST_FRAMEWORK]", context.test_framework.as_deref().unwrap_or("Unknown"));
        
        content.push_str(&zone1);
        
        // Lazy-load guidance
        if include_guidance && parts.len() > 1 {
            content.push_str("\nEXPERT_DEV_GUIDANCE:\n");
            content.push_str(parts[1].trim());
            content.push_str("\n");
        }
    }

    // 2. Load Language-specific rules
    if let Some(lang) = &context.language {
        let filename = format!("{}.toon", lang);
        if let Some(lang_rules) = Asset::get(&filename) {
            let lang_str = std::str::from_utf8(lang_rules.data.as_ref())?;
            content.push_str("\nZONE 2: DYNAMIC RULES (Top 3-5)\n");
            
            if let Ok(json) = serde_json::from_str::<JsonValue>(lang_str) {
                let mut selected_rules = Vec::new();

                // Framework-specific selection
                if let Some(framework) = &context.framework {
                    if let Some(arch_map) = json.get("architecture_map") {
                        if let Some(rule_ids) = arch_map.get(framework) {
                            if let Some(ids) = rule_ids.as_array() {
                                for id in ids {
                                    if let Some(id_str) = id.as_str() {
                                        selected_rules.push(id_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // Intent-based filtering (simplified heuristic)
                if let Some(intent_str) = intent {
                    let intent_lower = intent_str.to_lowercase();
                    if let Some(registry) = json.get("rule_registry").and_then(|v| v.as_array()) {
                        for rule in registry {
                            if let (Some(id), Some(trigger)) = (rule.get("id").and_then(|v| v.as_str()), rule.get("trigger").and_then(|v| v.as_str())) {
                                if trigger.to_lowercase().contains(&intent_lower) && !selected_rules.contains(&id.to_string()) {
                                    selected_rules.push(id.to_string());
                                }
                            }
                        }
                    }
                }

                // Limit to top 3-5 rules
                let limit = if selected_rules.len() > 5 { 5 } else { selected_rules.len() };
                let final_rules = &selected_rules[0..limit];

                if let Some(registry) = json.get("rule_registry").and_then(|v| v.as_array()) {
                    for id in final_rules {
                        if let Some(rule) = registry.iter().find(|r| r.get("id").and_then(|v| v.as_str()) == Some(id)) {
                            content.push_str(&format!("Rule {}:\n", id));
                            if let Some(trigger) = rule.get("trigger").and_then(|v| v.as_str()) {
                                content.push_str(&format!("  Trigger: {}\n", trigger));
                            }
                            if let Some(behavior) = rule.get("behavior").and_then(|v| v.as_str()) {
                                content.push_str(&format!("  Behavior: {}\n", behavior));
                            }
                            content.push_str("\n");
                        }
                    }
                }
            } else {
                // Fallback for non-JSON .toon files
                content.push_str(lang_str);
            }
        }
    }

    fs::write(output_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ProjectContext;
    use tempfile::tempdir;

    #[test]
    fn test_synthesize_rules_basic() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join(".assistant_rules.toon");
        let context = ProjectContext {
            language: Some("rust".to_string()),
            framework: Some("Command_Line_Interfaces".to_string()),
            build_system: Some("cargo".to_string()),
            test_framework: Some("cargo test".to_string()),
        };

        synthesize_rules(&context, &output_path, None, false).unwrap();
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("ZONE 1: OPERATIONAL FACTS"));
        assert!(content.contains("Build system: cargo"));
        assert!(content.contains("ZONE 2: DYNAMIC RULES"));
        // Should contain R01, R03, R07 based on architecture_map
        assert!(content.contains("Rule R01:"));
        assert!(content.contains("Rule R03:"));
        assert!(content.contains("Rule R07:"));
        // Should NOT contain guidance
        assert!(!content.contains("EXPERT_DEV_GUIDANCE:"));
    }

    #[test]
    fn test_synthesize_rules_with_guidance() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join(".assistant_rules.toon");
        let context = ProjectContext::default();

        synthesize_rules(&context, &output_path, None, true).unwrap();
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("EXPERT_DEV_GUIDANCE:"));
        assert!(content.contains("Prioritize composition over inheritance."));
    }

    #[test]
    fn test_synthesize_rules_with_intent() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join(".assistant_rules.toon");
        let context = ProjectContext {
            language: Some("rust".to_string()),
            ..ProjectContext::default()
        };

        // Intent "async" should pick up R02
        synthesize_rules(&context, &output_path, Some("async"), false).unwrap();
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Rule R02:"));
        assert!(content.contains("Trigger: async state, Tokio locks"));
    }

    #[test]
    fn test_synthesize_rules_ranking_limit() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join(".assistant_rules.toon");
        let context = ProjectContext {
            language: Some("rust".to_string()),
            framework: Some("Asynchronous_Web_Microservices".to_string()), // R01, R02, R03, R04
            ..ProjectContext::default()
        };

        // Adding intent "parallel" should add R06, making it 5 rules
        synthesize_rules(&context, &output_path, Some("parallel"), false).unwrap();
        
        let content = fs::read_to_string(output_path).unwrap();
        let rule_count = content.matches("Rule R").count();
        assert!(rule_count <= 5);
        assert!(content.contains("Rule R06:"));
    }
}
