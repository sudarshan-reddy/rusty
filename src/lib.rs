//! A high-performance MCP (Model Context Protocol) client for Neovim
//!
//! This library provides a robust, async MCP client that's compatible with
//! existing MCP server configurations from MCPHub, VS Code, Claude Desktop, etc.
//!
//! # Features
//!
//! - **Configuration Compatibility**: Uses the same config format as MCPHub, VS Code, Claude Desktop
//! - **Multiple Transports**: Supports stdio (child process) and HTTP/SSE transports
//! - **Concurrent Operations**: Built on tokio for high-performance async operations
//! - **Error Handling**: Comprehensive error handling and recovery
//! - **Extensible**: Easy to extend for custom MCP server types
//!
//! # Example
//!
//! ```rust,no_run
//! use nvim_mcp_client::{MCPClient, ConfigLoader};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration (compatible with MCPHub/VS Code format)
//!     let config = ConfigLoader::new().load()?;
//!     
//!     // Initialize client
//!     let mut client = MCPClient::new(config);
//!     client.initialize().await?;
//!     
//!     // List available tools
//!     let tools = client.list_all_tools().await?;
//!     println!("Available tools: {:#?}", tools);
//!     
//!     // Call a tool
//!     let result = client.call_tool(
//!         "filesystem",
//!         "read_file",
//!         Some(serde_json::json!({"path": "/tmp/test.txt"}))
//!     ).await?;
//!     
//!     println!("Tool result: {:#?}", result);
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod completion;
pub mod config;
pub mod providers;
pub mod server;

pub use client::{
    ConnectionStatus, MCPClient, MCPServerConnection, MCPService, Resource, ResourceContent, Tool,
    ToolResult, ToolResultContent,
};
pub use completion::{
    Completion, CompletionEngine, CompletionProvider, CompletionRequest, CompletionResponse,
    CompletionSource, Pattern, PatternDetector, Position,
};
pub use config::{ConfigLoader, MCPConfig, ServerConfig};
pub use server::JsonRpcServer;

// Re-export commonly used types
pub use anyhow::{Error, Result};
pub use serde_json::Value;
