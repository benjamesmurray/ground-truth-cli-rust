use rust_embed::RustEmbed;
use crate::scanner::ProjectContext;
use std::fs;
use std::path::Path;
use serde_json::Value as JsonValue;

#[derive(RustEmbed)]
#[folder = "rules/"]
struct Asset;

pub fn synthesize_rules(context: &ProjectContext, output_path: &Path) -> anyhow::Result<()> {
    let mut content = String::new();

    // 1. Load ground_truth_rules.toon
    if let Some(core_rules) = Asset::get("ground_truth_rules.toon") {
        let core_str = std::str::from_utf8(core_rules.data.as_ref())?;
        
        // Replace placeholders in Zone 1
        let mut zone1 = core_str.to_string();
        zone1 = zone1.replace("[CONTEXT_LIMIT]", "200,000 tokens");
        zone1 = zone1.replace("[BUILD_SYSTEM]", context.build_system.as_deref().unwrap_or("Unknown"));
        zone1 = zone1.replace("[TEST_FRAMEWORK]", context.test_framework.as_deref().unwrap_or("Unknown"));
        
        content.push_str(&zone1);
        content.push_str("\n");
    }

    // 2. Load Language-specific rules
    if let Some(lang) = &context.language {
        let filename = format!("{}.toon", lang);
        if let Some(lang_rules) = Asset::get(&filename) {
            let lang_str = std::str::from_utf8(lang_rules.data.as_ref())?;
            content.push_str("\nZONE 2: LANGUAGE-SPECIFIC RULES\n");
            
            if let Ok(json) = serde_json::from_str::<JsonValue>(lang_str) {
                // If we have a specific framework/architecture, filter or highlight those rules
                if let Some(framework) = &context.framework {
                    if let Some(arch_map) = json.get("architecture_map") {
                        if let Some(rule_ids) = arch_map.get(framework) {
                            content.push_str(&format!("Profile: {}\n", framework));
                            content.push_str("Selected Rules: ");
                            let ids: Vec<String> = rule_ids.as_array()
                                .unwrap_or(&vec![])
                                .iter()
                                .map(|v| v.as_str().unwrap_or("").to_string())
                                .collect();
                            content.push_str(&ids.join(", "));
                            content.push_str("\n\n");
                        }
                    }
                }
                
                // Append the raw language rules for full context
                content.push_str(lang_str);
            } else {
                content.push_str(lang_str);
            }
        }
    }

    fs::write(output_path, content)?;
    Ok(())
}
