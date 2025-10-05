//! JSON-RPC server for Neovim integration
//!
//! This module implements a JSON-RPC server that communicates with Neovim
//! over stdio. It handles completion requests and other MCP-related operations.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info, warn};

use crate::client::MCPClient;
use crate::completion::{CompletionEngine, CompletionRequest};

/// JSON-RPC request
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcError {
    fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        }
    }

    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    fn internal_error(message: String) -> Self {
        Self {
            code: -32603,
            message,
            data: None,
        }
    }
}

/// JSON-RPC server for handling Neovim requests
pub struct JsonRpcServer {
    completion_engine: CompletionEngine,
    mcp_client: Option<MCPClient>,
}

impl JsonRpcServer {
    /// Create a new JSON-RPC server
    pub fn new(completion_engine: CompletionEngine, mcp_client: Option<MCPClient>) -> Self {
        Self {
            completion_engine,
            mcp_client,
        }
    }

    /// Run the JSON-RPC server (blocking)
    ///
    /// This will read from stdin and write to stdout in a loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting JSON-RPC server on stdio");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to read line from stdin: {}", e);
                    break;
                }
            };

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            debug!("Received request: {}", line);

            // Parse and handle request
            let response = self.handle_request_line(&line).await;

            // Serialize and send response
            let response_json = serde_json::to_string(&response)?;
            debug!("Sending response: {}", response_json);

            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        info!("JSON-RPC server shutting down");
        Ok(())
    }

    /// Handle a single request line
    async fn handle_request_line(&mut self, line: &str) -> JsonRpcResponse {
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError::parse_error()),
                };
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError::invalid_request()),
            };
        }

        // Handle the request
        self.handle_request(request).await
    }

    /// Handle a parsed JSON-RPC request
    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "get_completion" => self.handle_get_completion(&request.params).await,
            "list_tools" => self.handle_list_tools().await,
            "list_resources" => self.handle_list_resources().await,
            "call_tool" => self.handle_call_tool(&request.params).await,
            "read_resource" => self.handle_read_resource(&request.params).await,
            "status" => self.handle_status().await,
            "shutdown" => {
                info!("Received shutdown request");
                Ok(serde_json::json!({"status": "shutting down"}))
            }
            _ => Err(anyhow!("Method not found: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => {
                warn!("Request failed: {}", e);
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(if e.to_string().starts_with("Method not found") {
                        JsonRpcError::method_not_found(&request.method)
                    } else {
                        JsonRpcError::internal_error(e.to_string())
                    }),
                }
            }
        }
    }

    /// Handle get_completion request
    async fn handle_get_completion(&self, params: &Value) -> Result<Value> {
        let request: CompletionRequest = serde_json::from_value(params.clone())?;

        debug!(
            "Completion request for {}:{} ({})",
            request.file_path, request.cursor_position.line, request.language
        );

        let response = self.completion_engine.get_completions(&request).await?;

        Ok(serde_json::to_value(response)?)
    }

    /// Handle list_tools request
    async fn handle_list_tools(&self) -> Result<Value> {
        if let Some(client) = &self.mcp_client {
            let tools = client.list_all_tools().await?;
            Ok(serde_json::to_value(tools)?)
        } else {
            Ok(serde_json::json!({}))
        }
    }

    /// Handle list_resources request
    async fn handle_list_resources(&self) -> Result<Value> {
        if let Some(client) = &self.mcp_client {
            let resources = client.list_all_resources().await?;
            Ok(serde_json::to_value(resources)?)
        } else {
            Ok(serde_json::json!({}))
        }
    }

    /// Handle call_tool request
    async fn handle_call_tool(&self, params: &Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct CallToolParams {
            server: String,
            tool: String,
            #[serde(default)]
            arguments: Option<Value>,
        }

        let params: CallToolParams = serde_json::from_value(params.clone())?;

        if let Some(client) = &self.mcp_client {
            let result = client
                .call_tool(&params.server, &params.tool, params.arguments)
                .await?;
            Ok(serde_json::to_value(result)?)
        } else {
            Err(anyhow!("MCP client not initialized"))
        }
    }

    /// Handle read_resource request
    async fn handle_read_resource(&self, params: &Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct ReadResourceParams {
            server: String,
            uri: String,
        }

        let params: ReadResourceParams = serde_json::from_value(params.clone())?;

        if let Some(client) = &self.mcp_client {
            let content = client.read_resource(&params.server, &params.uri).await?;
            Ok(serde_json::to_value(content)?)
        } else {
            Err(anyhow!("MCP client not initialized"))
        }
    }

    /// Handle status request
    async fn handle_status(&self) -> Result<Value> {
        let mut status = serde_json::json!({
            "completion_engine": "ready",
        });

        if let Some(client) = &self.mcp_client {
            let server_status = client.get_server_status();
            status["mcp_servers"] = serde_json::to_value(server_status)?;
        }

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_error_codes() {
        assert_eq!(JsonRpcError::parse_error().code, -32700);
        assert_eq!(JsonRpcError::invalid_request().code, -32600);
        assert_eq!(JsonRpcError::method_not_found("test").code, -32601);
    }

    #[test]
    fn test_jsonrpc_request_parsing() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"test","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "test");
        assert_eq!(request.jsonrpc, "2.0");
    }
}
