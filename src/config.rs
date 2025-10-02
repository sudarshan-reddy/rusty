use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// MCP Server configuration compatible with VS Code, MCPHub, Claude Desktop, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: IndexMap<String, ServerConfig>,
}

/// Individual server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerConfig {
    /// Local server with command and args (stdio transport)
    Local {
        command: String,
        args: Option<Vec<String>>,
        env: Option<HashMap<String, String>>,
        disabled: Option<bool>,
    },
    /// Remote server with URL (HTTP/SSE transport)
    Remote {
        url: String,
        headers: Option<HashMap<String, String>>,
        disabled: Option<bool>,
    },
}

impl ServerConfig {
    pub fn is_disabled(&self) -> bool {
        match self {
            ServerConfig::Local { disabled, .. } => disabled.unwrap_or(false),
            ServerConfig::Remote { disabled, .. } => disabled.unwrap_or(false),
        }
    }
}

/// Configuration loader with support for multiple config locations
pub struct ConfigLoader {
    search_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            search_paths: Self::default_search_paths(),
        }
    }

    /// Default search paths compatible with existing tools
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Current directory project configs
        paths.push(PathBuf::from(".mcphub/servers.json"));
        paths.push(PathBuf::from(".vscode/mcp.json"));
        paths.push(PathBuf::from(".cursor/mcp.json"));

        // User home directory configs
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".config/mcphub/servers.json"));
            paths.push(home.join(".config/mcp/servers.json"));
            paths.push(home.join("mcp/servers.json"));
        }

        // macOS specific paths
        if cfg!(target_os = "macos") {
            if let Some(home) = dirs::home_dir() {
                paths.push(
                    home.join("Library/Application Support/Claude/claude_desktop_config.json"),
                );
            }
        }

        paths
    }

    /// Add a custom search path
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Load configuration from the first available file
    pub fn load(&self) -> Result<MCPConfig> {
        for path in &self.search_paths {
            if path.exists() {
                tracing::info!("Loading MCP config from: {}", path.display());
                return self.load_from_file(path);
            }
        }

        tracing::warn!("No MCP configuration found in search paths");
        Ok(MCPConfig {
            mcp_servers: IndexMap::new(),
        })
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<MCPConfig> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file {}: {}", path.display(), e))?;

        // Expand environment variables and special syntax
        let expanded_content = self.expand_variables(&content)?;

        // Try to parse as MCP config first
        if let Ok(config) = serde_json::from_str::<MCPConfig>(&expanded_content) {
            return Ok(config);
        }

        // Try to parse as Claude Desktop config format
        if let Ok(claude_config) = serde_json::from_str::<ClaudeDesktopConfig>(&expanded_content) {
            return Ok(claude_config.into());
        }

        // Try to parse as VS Code format with "servers" key
        if let Ok(vscode_config) = serde_json::from_str::<VSCodeConfig>(&expanded_content) {
            return Ok(vscode_config.into());
        }

        Err(anyhow!("Failed to parse config file as any known format"))
    }

    /// Expand environment variables and special syntax like ${env:VAR}, ${input:prompt}
    fn expand_variables(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();

        // Expand ${env:VARIABLE_NAME} syntax
        let env_regex = regex::Regex::new(r"\$\{env:([^}]+)\}").unwrap();
        for cap in env_regex.captures_iter(content) {
            let var_name = &cap[1];
            let replacement = env::var(var_name).unwrap_or_else(|_| {
                tracing::warn!("Environment variable {} not found", var_name);
                String::new()
            });
            result = result.replace(&cap[0], &replacement);
        }

        // Expand home directory (~)
        result = shellexpand::tilde(&result).to_string();

        Ok(result)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Claude Desktop configuration format
#[derive(Debug, Deserialize)]
struct ClaudeDesktopConfig {
    #[serde(rename = "mcpServers")]
    mcp_servers: IndexMap<String, ServerConfig>,
}

impl From<ClaudeDesktopConfig> for MCPConfig {
    fn from(config: ClaudeDesktopConfig) -> Self {
        MCPConfig {
            mcp_servers: config.mcp_servers,
        }
    }
}

/// VS Code configuration format with "servers" key
#[derive(Debug, Deserialize)]
struct VSCodeConfig {
    servers: IndexMap<String, ServerConfig>,
}

impl From<VSCodeConfig> for MCPConfig {
    fn from(config: VSCodeConfig) -> Self {
        MCPConfig {
            mcp_servers: config.servers,
        }
    }
}

/// Configuration validation
impl MCPConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        for (name, server) in &self.mcp_servers {
            match server {
                ServerConfig::Local { command, .. } => {
                    if command.is_empty() {
                        return Err(anyhow!("Server '{}' has empty command", name));
                    }

                    // Check if command exists in PATH
                    if which::which(command).is_err() {
                        tracing::warn!(
                            "Command '{}' for server '{}' not found in PATH",
                            command,
                            name
                        );
                    }
                }
                ServerConfig::Remote { url, .. } => {
                    if url.is_empty() {
                        return Err(anyhow!("Server '{}' has empty URL", name));
                    }

                    // Basic URL validation
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return Err(anyhow!("Server '{}' has invalid URL: {}", name, url));
                    }
                }
            }
        }
        Ok(())
    }

    /// Get enabled servers only
    pub fn enabled_servers(&self) -> impl Iterator<Item = (&String, &ServerConfig)> {
        self.mcp_servers
            .iter()
            .filter(|(_, config)| !config.is_disabled())
    }

    /// Create a sample configuration file
    pub fn create_sample_config() -> Self {
        let mut servers = IndexMap::new();

        servers.insert(
            "filesystem".to_string(),
            ServerConfig::Local {
                command: "npx".to_string(),
                args: Some(vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    "/tmp".to_string(),
                ]),
                env: None,
                disabled: Some(false),
            },
        );

        servers.insert(
            "fetch".to_string(),
            ServerConfig::Local {
                command: "uvx".to_string(),
                args: Some(vec!["mcp-server-fetch".to_string()]),
                env: None,
                disabled: Some(false),
            },
        );

        servers.insert(
            "github".to_string(),
            ServerConfig::Remote {
                url: "https://api.githubcopilot.com/mcp/".to_string(),
                headers: Some({
                    let mut headers = HashMap::new();
                    headers.insert(
                        "Authorization".to_string(),
                        "Bearer ${env:GITHUB_PERSONAL_ACCESS_TOKEN}".to_string(),
                    );
                    headers
                }),
                disabled: Some(true),
            },
        );

        MCPConfig {
            mcp_servers: servers,
        }
    }
}
