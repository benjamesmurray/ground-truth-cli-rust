mod scanner;
mod rules;

use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::scanner::scan_project;
use crate::rules::synthesize_rules;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: serde_json::Value,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(_) => continue,
        };

        let response = handle_request(request).await;
        let response_str = serde_json::to_string(&response)? + "\n";
        stdout.write_all(response_str.as_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.unwrap_or(json!(null));
    
    match request.method.as_str() {
        "initialize" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "ground-truth-cli-rust",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            }
        },
        "tools/list" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "gt_status",
                            "description": "Returns the current project state and detection results.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "gt_exec_scan",
                            "description": "Runs the scanner and generates the .assistant_rules.toon file.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {
                                        "type": "string",
                                        "description": "The path to scan (defaults to current directory)."
                                    }
                                }
                            }
                        }
                    ]
                })),
                error: None,
            }
        },
        "tools/call" => {
            let tool_name = request.params.as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            
            let args = request.params.as_ref()
                .and_then(|p| p.get("arguments"))
                .cloned()
                .unwrap_or(json!({}));

            match tool_name {
                "gt_status" => {
                    let context = scan_project(Path::new("."));
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Project Status:\nLanguage: {:?}\nFramework: {:?}\nBuild System: {:?}\nTest Framework: {:?}", 
                                        context.language, context.framework, context.build_system, context.test_framework)
                                }
                            ]
                        })),
                        error: None,
                    }
                },
                "gt_exec_scan" => {
                    let scan_path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
                    let context = scan_project(Path::new(scan_path));
                    let output_path = Path::new(scan_path).join(".assistant_rules.toon");
                    
                    match synthesize_rules(&context, &output_path) {
                        Ok(_) => {
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: Some(json!({
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": format!("Successfully scanned project and generated {}", output_path.display())
                                        }
                                    ]
                                })),
                                error: None,
                            }
                        },
                        Err(e) => {
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(json!({
                                    "code": -32000,
                                    "message": format!("Failed to synthesize rules: {}", e)
                                })),
                            }
                        }
                    }
                },
                _ => {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32601,
                            "message": "Method not found"
                        })),
                    }
                }
            }
        },
        _ => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": "Method not found"
                })),
            }
        }
    }
}
