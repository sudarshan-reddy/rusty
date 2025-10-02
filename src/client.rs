use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use rmcp::{
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::TokioChildProcess,
};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

use crate::config::{MCPConfig, ServerConfig};

/// A connection to a single MCP server
pub struct MCPServerConnection {
    pub name: String,
    pub config: ServerConfig,
    pub service: Option<Box<dyn MCPService>>,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

/// Trait for abstracting MCP service operations
#[async_trait::async_trait]
pub trait MCPService: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<Tool>>;
    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<ToolResult>;
    async fn list_resources(&self) -> Result<Vec<Resource>>;
    async fn read_resource(&self, uri: &str) -> Result<ResourceContent>;
    async fn disconnect(&mut self) -> Result<()>;
}

/// Tool information
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub content: Vec<ToolResultContent>,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct ToolResultContent {
    pub content_type: String,
    pub text: Option<String>,
    pub data: Option<Value>,
}

/// Resource information
#[derive(Debug, Clone)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Resource content
#[derive(Debug, Clone)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}

/// Main MCP client that manages multiple server connections
pub struct MCPClient {
    connections: IndexMap<String, MCPServerConnection>,
    config: MCPConfig,
}

impl MCPClient {
    /// Create a new MCP client with the given configuration
    pub fn new(config: MCPConfig) -> Self {
        Self {
            connections: IndexMap::new(),
            config,
        }
    }

    /// Initialize all configured servers
    pub async fn initialize(&mut self) -> Result<()> {
        info!(
            "Initializing MCP client with {} servers",
            self.config.mcp_servers.len()
        );

        // Validate configuration first
        self.config.validate()?;

        // Initialize each enabled server
        for (name, server_config) in self.config.enabled_servers() {
            let connection = MCPServerConnection {
                name: name.clone(),
                config: server_config.clone(),
                service: None,
                status: ConnectionStatus::Disconnected,
            };

            self.connections.insert(name.clone(), connection);
        }

        // Connect to all servers concurrently
        self.connect_all().await?;

        info!("MCP client initialization complete");
        Ok(())
    }

    /// Connect to all configured servers
    pub async fn connect_all(&mut self) -> Result<()> {
        let mut connection_tasks = Vec::new();

        for (name, connection) in &mut self.connections {
            if matches!(connection.status, ConnectionStatus::Disconnected) {
                connection.status = ConnectionStatus::Connecting;

                let server_config = connection.config.clone();
                let server_name = name.clone();

                let task = tokio::spawn(async move {
                    Self::create_service_for_config(&server_name, &server_config).await
                });

                connection_tasks.push((name.clone(), task));
            }
        }

        // Wait for all connections to complete
        for (name, task) in connection_tasks {
            match task.await {
                Ok(Ok(service)) => {
                    if let Some(connection) = self.connections.get_mut(&name) {
                        connection.service = Some(service);
                        connection.status = ConnectionStatus::Connected;
                        info!("Successfully connected to MCP server: {}", name);
                    }
                }
                Ok(Err(e)) => {
                    if let Some(connection) = self.connections.get_mut(&name) {
                        connection.status = ConnectionStatus::Failed(e.to_string());
                        error!("Failed to connect to MCP server {}: {}", name, e);
                    }
                }
                Err(e) => {
                    if let Some(connection) = self.connections.get_mut(&name) {
                        connection.status = ConnectionStatus::Failed(format!("Task failed: {}", e));
                        error!("Connection task failed for {}: {}", name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a service for a given server configuration
    async fn create_service_for_config(
        name: &str,
        config: &ServerConfig,
    ) -> Result<Box<dyn MCPService>> {
        match config {
            ServerConfig::Local {
                command, args, env, ..
            } => {
                debug!(
                    "Creating local MCP service for {}: {} {:?}",
                    name, command, args
                );

                let mut cmd = Command::new(command);

                // Add arguments
                if let Some(args) = args {
                    cmd.args(args);
                }

                // Set environment variables
                if let Some(env_vars) = env {
                    for (key, value) in env_vars {
                        cmd.env(key, value);
                    }
                }

                // Configure for MCP stdio transport
                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                let transport = TokioChildProcess::new(cmd)?;
                let service = LocalMCPService::new(transport).await?;

                Ok(Box::new(service))
            }
            ServerConfig::Remote { url, .. } => {
                debug!("Creating remote MCP service for {}: {}", name, url);

                // TODO: Implement HTTP/SSE transport for remote servers
                // This would use rmcp's HTTP transport capabilities

                Err(anyhow!("Remote MCP servers not yet implemented"))
            }
        }
    }

    /// Get all available tools from all connected servers
    pub async fn list_all_tools(&self) -> Result<HashMap<String, Vec<Tool>>> {
        let mut all_tools = HashMap::new();

        for (name, connection) in &self.connections {
            if let (Some(service), ConnectionStatus::Connected) =
                (&connection.service, &connection.status)
            {
                match service.list_tools().await {
                    Ok(tools) => {
                        debug!("Server {} has {} tools", name, tools.len());
                        all_tools.insert(name.clone(), tools);
                    }
                    Err(e) => {
                        warn!("Failed to list tools for server {}: {}", name, e);
                    }
                }
            }
        }

        Ok(all_tools)
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> Result<ToolResult> {
        let connection = self
            .connections
            .get(server_name)
            .ok_or_else(|| anyhow!("Server '{}' not found", server_name))?;

        if !matches!(connection.status, ConnectionStatus::Connected) {
            return Err(anyhow!("Server '{}' is not connected", server_name));
        }

        let service = connection
            .service
            .as_ref()
            .ok_or_else(|| anyhow!("Server '{}' has no service", server_name))?;

        service.call_tool(tool_name, arguments).await
    }

    /// Get all available resources from all connected servers
    pub async fn list_all_resources(&self) -> Result<HashMap<String, Vec<Resource>>> {
        let mut all_resources = HashMap::new();

        for (name, connection) in &self.connections {
            if let (Some(service), ConnectionStatus::Connected) =
                (&connection.service, &connection.status)
            {
                match service.list_resources().await {
                    Ok(resources) => {
                        debug!("Server {} has {} resources", name, resources.len());
                        all_resources.insert(name.clone(), resources);
                    }
                    Err(e) => {
                        warn!("Failed to list resources for server {}: {}", name, e);
                    }
                }
            }
        }

        Ok(all_resources)
    }

    /// Read a resource from a specific server
    pub async fn read_resource(
        &self,
        server_name: &str,
        resource_uri: &str,
    ) -> Result<ResourceContent> {
        let connection = self
            .connections
            .get(server_name)
            .ok_or_else(|| anyhow!("Server '{}' not found", server_name))?;

        if !matches!(connection.status, ConnectionStatus::Connected) {
            return Err(anyhow!("Server '{}' is not connected", server_name));
        }

        let service = connection
            .service
            .as_ref()
            .ok_or_else(|| anyhow!("Server '{}' has no service", server_name))?;

        service.read_resource(resource_uri).await
    }

    /// Get connection status for all servers
    pub fn get_server_status(&self) -> HashMap<String, ConnectionStatus> {
        self.connections
            .iter()
            .map(|(name, conn)| (name.clone(), conn.status.clone()))
            .collect()
    }

    /// Reconnect to a specific server
    pub async fn reconnect_server(&mut self, server_name: &str) -> Result<()> {
        let connection = self
            .connections
            .get_mut(server_name)
            .ok_or_else(|| anyhow!("Server '{}' not found", server_name))?;

        // Disconnect existing service if any
        if let Some(mut service) = connection.service.take() {
            let _ = service.disconnect().await;
        }

        connection.status = ConnectionStatus::Connecting;

        match Self::create_service_for_config(server_name, &connection.config).await {
            Ok(service) => {
                connection.service = Some(service);
                connection.status = ConnectionStatus::Connected;
                info!("Successfully reconnected to server: {}", server_name);
                Ok(())
            }
            Err(e) => {
                connection.status = ConnectionStatus::Failed(e.to_string());
                Err(e)
            }
        }
    }

    /// Shutdown all connections
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down MCP client");

        for (name, connection) in &mut self.connections {
            if let Some(mut service) = connection.service.take() {
                if let Err(e) = service.disconnect().await {
                    warn!("Error disconnecting from server {}: {}", name, e);
                }
            }
            connection.status = ConnectionStatus::Disconnected;
        }

        Ok(())
    }
}

/// Implementation of MCPService for local (stdio) servers using rmcp
struct LocalMCPService {
    service: rmcp::service::RunningService<rmcp::service::RoleClient, ()>,
}

impl LocalMCPService {
    async fn new(transport: TokioChildProcess) -> Result<Self> {
        let service = ().serve(transport).await?;
        Ok(Self { service })
    }
}

#[async_trait::async_trait]
impl MCPService for LocalMCPService {
    async fn list_tools(&self) -> Result<Vec<Tool>> {
        let response = self
            .service
            .list_tools(None)
            .await?;

        let tools = response
            .tools
            .into_iter()
            .map(|tool| Tool {
                name: tool.name.to_string(),
                description: tool.description.map(|d| d.to_string()),
                input_schema: Some(serde_json::Value::Object((*tool.input_schema).clone())),
            })
            .collect();

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<ToolResult> {
        let param = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments.and_then(|v| v.as_object().cloned()),
        };

        let response = self.service.call_tool(param).await?;

        let content = response
            .content
            .into_iter()
            .map(|content| {
                let (content_type, text, data) = match &content.raw {
                    rmcp::model::RawContent::Text(text_content) => {
                        ("text".to_string(), Some(text_content.text.clone()), None)
                    }
                    rmcp::model::RawContent::Image(image_content) => {
                        ("image".to_string(), None, Some(serde_json::json!({
                            "data": image_content.data,
                            "mime_type": image_content.mime_type
                        })))
                    }
                    rmcp::model::RawContent::Resource(resource) => {
                        ("resource".to_string(), None, Some(serde_json::to_value(resource).unwrap_or_default()))
                    }
                    rmcp::model::RawContent::Audio(audio) => {
                        ("audio".to_string(), None, Some(serde_json::to_value(audio).unwrap_or_default()))
                    }
                };

                ToolResultContent {
                    content_type,
                    text,
                    data,
                }
            })
            .collect();

        Ok(ToolResult {
            content,
            is_error: response.is_error.unwrap_or(false),
        })
    }

    async fn list_resources(&self) -> Result<Vec<Resource>> {
        let response = self
            .service
            .list_resources(None)
            .await?;

        let resources = response
            .resources
            .into_iter()
            .map(|resource| Resource {
                uri: resource.raw.uri.clone(),
                name: resource.raw.name.clone(),
                description: resource.raw.description.clone(),
                mime_type: resource.raw.mime_type.clone(),
            })
            .collect();

        Ok(resources)
    }

    async fn read_resource(&self, uri: &str) -> Result<ResourceContent> {
        let response = self
            .service
            .read_resource(rmcp::model::ReadResourceRequestParam {
                uri: uri.to_string(),
            })
            .await?;

        // Handle different content types
        let (text, blob) = if let Some(contents) = response.contents.first() {
            match contents {
                rmcp::model::ResourceContents::TextResourceContents { text, .. } => (Some(text.clone()), None),
                rmcp::model::ResourceContents::BlobResourceContents { blob, .. } => (None, Some(blob.clone())),
            }
        } else {
            (None, None)
        };

        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: response.contents.first().and_then(|c| match c {
                rmcp::model::ResourceContents::TextResourceContents { mime_type, .. } => mime_type.clone(),
                rmcp::model::ResourceContents::BlobResourceContents { mime_type, .. } => mime_type.clone(),
            }),
            text,
            blob,
        })
    }

    async fn disconnect(&mut self) -> Result<()> {
        // The rmcp service will handle cleanup automatically when dropped
        Ok(())
    }
}

impl Drop for LocalMCPService {
    fn drop(&mut self) {
        // rmcp handles cleanup automatically
    }
}
