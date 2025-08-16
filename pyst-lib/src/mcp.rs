use crate::config::Config;
use crate::discovery::Discovery;
use crate::executor::Executor;
use crate::introspection::runner::IntrospectionRunner;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResource {
    uri: String,
    name: String,
    description: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    client_info: ClientInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientCapabilities {
    roots: Option<RootsCapability>,
    sampling: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RootsCapability {
    #[serde(rename = "listChanged")]
    list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerCapabilities {
    logging: Option<serde_json::Value>,
    prompts: Option<PromptsCapability>,
    resources: Option<ResourcesCapability>,
    tools: Option<ToolsCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptsCapability {
    #[serde(rename = "listChanged")]
    list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourcesCapability {
    subscribe: Option<bool>,
    #[serde(rename = "listChanged")]
    list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolsCapability {
    #[serde(rename = "listChanged")]
    list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    name: String,
    version: String,
}

pub struct McpServer {
    config: Config,
}

impl McpServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn start_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        eprintln!("MCP Server starting in stdio mode");
        
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            
            if bytes_read == 0 {
                break; // EOF
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            match self.handle_request(trimmed).await {
                Ok(response) => {
                    stdout.write_all(response.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    eprintln!("Error handling request: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_request(&self, request_str: &str) -> Result<String> {
        let request: JsonRpcRequest = serde_json::from_str(request_str)?;
        
        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "initialized" => self.handle_initialized(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tools_call(&request).await,
            "resources/list" => self.handle_resources_list(&request).await,
            "resources/read" => self.handle_resources_read(&request).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        };
        
        Ok(serde_json::to_string(&response)?)
    }
    
    async fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let capabilities = ServerCapabilities {
            logging: None,
            prompts: None,
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(false),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(false),
            }),
        };
        
        let server_info = ServerInfo {
            name: "pyst".to_string(),
            version: "0.1.0".to_string(),
        };
        
        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": capabilities,
            "serverInfo": server_info
        });
        
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(result),
            error: None,
        }
    }
    
    async fn handle_initialized(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(serde_json::json!({})),
            error: None,
        }
    }
    
    async fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let tools = vec![
            McpTool {
                name: "list_scripts".to_string(),
                description: "List all available Python scripts in the project".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "all": {
                            "type": "boolean",
                            "description": "Include disabled scripts",
                            "default": false
                        },
                        "context": {
                            "type": "string",
                            "description": "Context to use for filtering scripts"
                        }
                    }
                }),
            },
            McpTool {
                name: "run_script".to_string(),
                description: "Execute a Python script with optional arguments".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "Script name or path to execute"
                        },
                        "args": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Arguments to pass to the script",
                            "default": []
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force execution bypassing context rules",
                            "default": false
                        }
                    },
                    "required": ["script"]
                }),
            },
            McpTool {
                name: "get_script_info".to_string(),
                description: "Get detailed information about a specific script".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "Script name or path"
                        }
                    },
                    "required": ["script"]
                }),
            },
            McpTool {
                name: "explain_script".to_string(),
                description: "Explain why a script is enabled or disabled in the current context".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "Script name to explain"
                        },
                        "context": {
                            "type": "string",
                            "description": "Context to evaluate (optional)"
                        }
                    },
                    "required": ["script"]
                }),
            },
        ];
        
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(serde_json::json!({"tools": tools})),
            error: None,
        }
    }
    
    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = request.params.as_ref();
        if params.is_none() {
            return self.error_response(request.id.clone(), -32602, "Invalid params");
        }
        
        let params = params.unwrap();
        let tool_name = params.get("name").and_then(|v| v.as_str());
        let arguments = params.get("arguments");
        
        if tool_name.is_none() {
            return self.error_response(request.id.clone(), -32602, "Missing tool name");
        }
        
        let result = match tool_name.unwrap() {
            "list_scripts" => self.tool_list_scripts(arguments).await,
            "run_script" => self.tool_run_script(arguments).await,
            "get_script_info" => self.tool_get_script_info(arguments).await,
            "explain_script" => self.tool_explain_script(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name.unwrap())),
        };
        
        match result {
            Ok(content) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                })),
                error: None,
            },
            Err(e) => self.error_response(request.id.clone(), -32603, &e.to_string()),
        }
    }
    
    async fn handle_resources_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let resources = vec![
            McpResource {
                uri: "pyst://config".to_string(),
                name: "Pyst Configuration".to_string(),
                description: "Current pyst configuration including contexts and settings".to_string(),
                mime_type: "application/json".to_string(),
            },
            McpResource {
                uri: "pyst://project-info".to_string(),
                name: "Project Information".to_string(),
                description: "Information about the current project and discovered scripts".to_string(),
                mime_type: "application/json".to_string(),
            },
        ];
        
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(serde_json::json!({"resources": resources})),
            error: None,
        }
    }
    
    async fn handle_resources_read(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = request.params.as_ref();
        if params.is_none() {
            return self.error_response(request.id.clone(), -32602, "Invalid params");
        }
        
        let uri = params.unwrap().get("uri").and_then(|v| v.as_str());
        if uri.is_none() {
            return self.error_response(request.id.clone(), -32602, "Missing URI");
        }
        
        let content = match uri.unwrap() {
            "pyst://config" => serde_json::to_string_pretty(&self.config).unwrap_or_else(|_| "Error serializing config".to_string()),
            "pyst://project-info" => self.get_project_info().await.unwrap_or_else(|e| format!("Error: {}", e)),
            _ => "Resource not found".to_string(),
        };
        
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(serde_json::json!({
                "contents": [
                    {
                        "uri": uri.unwrap(),
                        "mimeType": "application/json",
                        "text": content
                    }
                ]
            })),
            error: None,
        }
    }
    
    async fn tool_list_scripts(&self, arguments: Option<&serde_json::Value>) -> Result<String> {
        let show_all = arguments
            .and_then(|a| a.get("all"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let context = arguments
            .and_then(|a| a.get("context"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        let discovery = Discovery::new(self.config.clone());
        let project_root = std::env::current_dir().ok();
        let scripts = discovery.discover_scripts(project_root.as_deref())?;
        
        let mut runner = IntrospectionRunner::new(self.config.clone())?;
        let script_paths: Vec<_> = scripts.iter().map(|s| s.path.clone()).collect();
        let introspection_results = runner.introspect_batch(&script_paths)?;
        
        let mut output = Vec::new();
        output.push(format!("📋 Available Scripts (context: {})", context));
        output.push("".to_string());
        
        for (script, introspection) in scripts.iter().zip(introspection_results.iter()) {
            let enabled = self.config.contexts.is_script_enabled(context, &script.name);
            
            if !show_all && !enabled {
                continue;
            }
            
            let scope = if script.is_local { "local" } else { "global" };
            let status = if enabled { "" } else { " (disabled)" };
            let description = introspection.metadata.description.as_deref().unwrap_or("");
            
            if description.is_empty() {
                output.push(format!("• {} ({}){}", script.name, scope, status));
            } else {
                output.push(format!("• {} ({}) - {}{}", script.name, scope, description, status));
            }
        }
        
        if output.len() == 2 {
            output.push("No scripts found".to_string());
        }
        
        Ok(output.join("\n"))
    }
    
    async fn tool_run_script(&self, arguments: Option<&serde_json::Value>) -> Result<String> {
        if arguments.is_none() {
            return Err(anyhow!("Missing arguments"));
        }
        
        let args = arguments.unwrap();
        let script = args.get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing script parameter"))?;
        
        let script_args: Vec<String> = args.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        
        let force = args.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // For MCP, we'll use a simplified execution that captures output
        // Use current executable instead of hardcoded debug path
        let pyst_exe = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("pyst"));
        let mut cmd = Command::new(pyst_exe);
        cmd.args(&["run", script]);
        
        if force {
            cmd.arg("--force");
        }
        
        if !script_args.is_empty() {
            cmd.arg("--");
            cmd.args(&script_args);
        }
        
        let output = cmd.output().await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let mut result = Vec::new();
        result.push(format!("🚀 Executed: {} {}", script, script_args.join(" ")));
        result.push("".to_string());
        
        if !stdout.is_empty() {
            result.push("📤 Output:".to_string());
            result.push(stdout.to_string());
        }
        
        if !stderr.is_empty() {
            result.push("⚠️ Error:".to_string());
            result.push(stderr.to_string());
        }
        
        result.push(format!("Exit code: {}", output.status.code().unwrap_or(-1)));
        
        Ok(result.join("\n"))
    }
    
    async fn tool_get_script_info(&self, arguments: Option<&serde_json::Value>) -> Result<String> {
        if arguments.is_none() {
            return Err(anyhow!("Missing arguments"));
        }
        
        let args = arguments.unwrap();
        let script = args.get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing script parameter"))?;
        
        let discovery = Discovery::new(self.config.clone());
        let project_root = std::env::current_dir().ok();
        let script_info = discovery.resolve_script(script, project_root.as_deref())?;
        
        let mut runner = IntrospectionRunner::new(self.config.clone())?;
        let introspection = runner.introspect(&script_info.path)?;
        
        let active_context = std::env::var("PYST_CONTEXT").unwrap_or_else(|_| "default".to_string());
        let enabled = self.config.contexts.is_script_enabled(&active_context, &script_info.name);
        let trusted = runner.is_trusted(&script_info.path);
        
        let mut output = Vec::new();
        output.push(format!("📄 Script Information: {}", script_info.name));
        output.push("".to_string());
        output.push(format!("Path: {}", script_info.path.display()));
        output.push(format!("Scope: {}", if script_info.is_local { "local" } else { "global" }));
        output.push(format!("Status: {}", if enabled { "enabled" } else { "disabled" }));
        output.push(format!("Trusted: {}", if trusted { "yes" } else { "no" }));
        output.push(format!("Entry Point: {:?}", script_info.entry_point));
        
        if let Some(desc) = &introspection.metadata.description {
            output.push(format!("Description: {}", desc));
        }
        
        if let Some(pep723) = &introspection.metadata.pep723_metadata {
            output.push("".to_string());
            output.push("PEP 723 Metadata:".to_string());
            output.push(format!("  Dependencies: {:?}", pep723.dependencies));
            if let Some(requires_python) = &pep723.requires_python {
                output.push(format!("  Requires Python: {}", requires_python));
            }
        }
        
        if !introspection.metadata.dependencies.is_empty() {
            output.push("".to_string());
            output.push("Dependencies:".to_string());
            for dep in &introspection.metadata.dependencies {
                let version = dep.version_spec.as_deref().unwrap_or("");
                output.push(format!("  • {} {} (from {:?})", dep.name, version, dep.source));
            }
        }
        
        if let Some(cli_framework) = &introspection.metadata.cli_framework {
            output.push("".to_string());
            output.push(format!("CLI Framework: {}", cli_framework.name));
            if let Some(version) = &cli_framework.version {
                output.push(format!("Framework Version: {}", version));
            }
        }
        
        Ok(output.join("\n"))
    }
    
    async fn tool_explain_script(&self, arguments: Option<&serde_json::Value>) -> Result<String> {
        if arguments.is_none() {
            return Err(anyhow!("Missing arguments"));
        }
        
        let args = arguments.unwrap();
        let script = args.get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing script parameter"))?;
        
        let context = args.get("context")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "default");
        
        let evaluation = self.config.contexts.evaluate_script(context, script);
        
        let mut output = Vec::new();
        output.push(format!("🔍 Script Context Evaluation: {}", evaluation.script_name));
        output.push("".to_string());
        output.push(format!("Context: {}", evaluation.context_name));
        output.push(format!("Status: {}", if evaluation.enabled { "✅ ENABLED" } else { "❌ DISABLED" }));
        output.push("".to_string());
        
        if let Some(final_rule) = &evaluation.final_rule {
            output.push("Final determining rule:".to_string());
            output.push(format!("  Pattern: {}", final_rule.pattern));
            output.push(format!("  Type: {}", if final_rule.is_negation { "Exclusion (!)" } else { "Inclusion" }));
            output.push(format!("  Matches: {}", final_rule.matches));
            output.push("".to_string());
        }
        
        output.push("All rules in context:".to_string());
        for (index, rule) in evaluation.all_rules.iter().enumerate() {
            let status = if rule.matches {
                if rule.is_negation { "MATCHES (disables)" } else { "MATCHES (enables)" }
            } else {
                "no match"
            };
            output.push(format!("  {}: {} -> {}", index + 1, rule.pattern, status));
        }
        
        if evaluation.matched_rules.len() > 1 {
            output.push("".to_string());
            output.push("Note: Multiple rules matched. The last matching rule takes precedence.".to_string());
        }
        
        Ok(output.join("\n"))
    }
    
    async fn get_project_info(&self) -> Result<String> {
        let discovery = Discovery::new(self.config.clone());
        let project_root = std::env::current_dir().ok();
        let scripts = discovery.discover_scripts(project_root.as_deref())?;
        
        let project_info = serde_json::json!({
            "project_root": project_root.as_ref().map(|p| p.display().to_string()),
            "total_scripts": scripts.len(),
            "local_scripts": scripts.iter().filter(|s| s.is_local).count(),
            "global_scripts": scripts.iter().filter(|s| !s.is_local).count(),
            "scripts": scripts.iter().map(|s| serde_json::json!({
                "name": s.name,
                "path": s.path.display().to_string(),
                "is_local": s.is_local,
                "entry_point": format!("{:?}", s.entry_point),
            })).collect::<Vec<_>>()
        });
        
        Ok(serde_json::to_string_pretty(&project_info)?)
    }
    
    fn error_response(&self, id: Option<serde_json::Value>, code: i32, message: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}