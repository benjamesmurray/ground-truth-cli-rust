mod scanner;
mod rules;

use async_trait::async_trait;
use rust_mcp_sdk::{
    *,
    error::SdkResult,
    macros,
    mcp_server::{server_runtime, ServerHandler, McpServerOptions},
    schema::*,
};
use crate::scanner::scan_project;
use crate::rules::synthesize_rules;
use std::path::Path;
use std::sync::Arc;

#[macros::mcp_tool(
    name = "gt_status",
    description = "Returns the current project state and detection results."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, macros::JsonSchema)]
pub struct GtStatusTool {}

#[macros::mcp_tool(
    name = "gt_exec_scan",
    description = "Runs the scanner and generates the .assistant_rules.toon file."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, macros::JsonSchema)]
pub struct GtExecScanTool {
    /// The path to scan (defaults to current directory).
    pub path: Option<String>,
}

#[derive(Default)]
struct GtHandler;

#[async_trait]
impl ServerHandler for GtHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: vec![GtStatusTool::tool(), GtExecScanTool::tool()],
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "gt_status" => {
                let context = scan_project(Path::new("."));
                Ok(CallToolResult::text_content(vec![
                    format!("Project Status:\nLanguage: {:?}\nFramework: {:?}\nBuild System: {:?}\nTest Framework: {:?}", 
                        context.language, context.framework, context.build_system, context.test_framework).into()
                ]))
            },
            "gt_exec_scan" => {
                let args: GtExecScanTool = serde_json::from_value(serde_json::Value::Object(params.arguments.unwrap_or_default()))
                    .map_err(|e| CallToolError::invalid_arguments("gt_exec_scan", Some(format!("Invalid arguments: {}", e))))?;
                
                let scan_path = args.path.as_deref().unwrap_or(".");
                let context = scan_project(Path::new(scan_path));
                let output_path = Path::new(scan_path).join(".assistant_rules.toon");
                
                match synthesize_rules(&context, &output_path) {
                    Ok(_) => {
                        Ok(CallToolResult::text_content(vec![
                            format!("Successfully scanned project and generated {}", output_path.display()).into()
                        ]))
                    },
                    Err(e) => {
                        Err(CallToolError::from_message(format!("Failed to synthesize rules: {}", e)))
                    }
                }
            },
            _ => Err(CallToolError::unknown_tool(params.name)),
        }
    }
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    let server_info = InitializeResult {
        server_info: Implementation {
            name: "ground-truth-cli-rust".into(),
            version: "0.1.2".into(),
            description: Some("A high-performance Rust reimplementation of the ground-truth-cli MCP server.".into()),
            icons: vec![],
            website_url: Some("https://github.com/benjamesmurray/ground-truth-cli-rust".into()),
            title: Some("Ground Truth CLI".into()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: Some(false) }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_11_25.into(),
        instructions: None,
        meta: None,
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = GtHandler::default().to_mcp_server_handler();
    
    let options = McpServerOptions {
        server_details: server_info,
        transport,
        handler,
        task_store: None,
        client_task_store: None,
        message_observer: None,
    };

    let server = server_runtime::create_server(options);
    
    server.start().await
}
