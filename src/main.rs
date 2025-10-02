use anyhow::Result;
use clap::{Arg, Command};
use std::io::{self, Write};
use tracing::{info, Level};
use tracing_subscriber;

// Import from our library crate
use nvim_mcp_client::{ConfigLoader, ConnectionStatus, MCPClient, MCPConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("rusty")
        .version("0.1.0")
        .author("Sudarsan Reddy")
        .about("A high-performance MCP client for Neovim")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path"),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (trace, debug, info, warn, error)")
                .default_value("info"),
        )
        .arg(
            Arg::new("create-config")
                .long("create-config")
                .value_name("FILE")
                .help("Create a sample configuration file"),
        )
        .subcommand(
            Command::new("list-tools").about("List all available tools from connected servers"),
        )
        .subcommand(
            Command::new("list-resources")
                .about("List all available resources from connected servers"),
        )
        .subcommand(
            Command::new("call-tool")
                .about("Call a specific tool")
                .arg(
                    Arg::new("server")
                        .short('s')
                        .long("server")
                        .value_name("SERVER")
                        .help("Server name")
                        .required(true),
                )
                .arg(
                    Arg::new("tool")
                        .short('t')
                        .long("tool")
                        .value_name("TOOL")
                        .help("Tool name")
                        .required(true),
                )
                .arg(
                    Arg::new("args")
                        .short('a')
                        .long("args")
                        .value_name("JSON")
                        .help("Tool arguments as JSON"),
                ),
        )
        .subcommand(
            Command::new("read-resource")
                .about("Read a specific resource")
                .arg(
                    Arg::new("server")
                        .short('s')
                        .long("server")
                        .value_name("SERVER")
                        .help("Server name")
                        .required(true),
                )
                .arg(
                    Arg::new("uri")
                        .short('u')
                        .long("uri")
                        .value_name("URI")
                        .help("Resource URI")
                        .required(true),
                ),
        )
        .subcommand(Command::new("status").about("Show connection status for all servers"))
        .subcommand(Command::new("interactive").about("Start interactive mode"))
        .get_matches();

    // Initialize logging
    let log_level = match matches.get_one::<String>("log-level").unwrap().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Handle create-config command
    if let Some(config_path) = matches.get_one::<String>("create-config") {
        create_sample_config(config_path)?;
        return Ok(());
    }

    // Load configuration
    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        ConfigLoader::new().load_from_file(config_path)?
    } else {
        ConfigLoader::new().load()?
    };

    info!(
        "Loaded configuration with {} servers",
        config.mcp_servers.len()
    );

    // Initialize MCP client
    let mut client = MCPClient::new(config);
    client.initialize().await?;

    // Handle subcommands
    match matches.subcommand() {
        Some(("list-tools", _)) => {
            list_tools(&client).await?;
        }
        Some(("list-resources", _)) => {
            list_resources(&client).await?;
        }
        Some(("call-tool", sub_matches)) => {
            let server = sub_matches.get_one::<String>("server").unwrap();
            let tool = sub_matches.get_one::<String>("tool").unwrap();
            let args = sub_matches.get_one::<String>("args");
            call_tool(&client, server, tool, args).await?;
        }
        Some(("read-resource", sub_matches)) => {
            let server = sub_matches.get_one::<String>("server").unwrap();
            let uri = sub_matches.get_one::<String>("uri").unwrap();
            read_resource(&client, server, uri).await?;
        }
        Some(("status", _)) => {
            show_status(&client).await?;
        }
        Some(("interactive", _)) => {
            interactive_mode(client).await?;
        }
        None => {
            // Default: show status and enter interactive mode
            show_status(&client).await?;
            println!("\nEntering interactive mode. Type 'help' for commands.");
            interactive_mode(client).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn list_tools(client: &MCPClient) -> Result<()> {
    let tools = client.list_all_tools().await?;

    if tools.is_empty() {
        println!("No tools available from connected servers.");
        return Ok(());
    }

    println!("Available Tools:");
    println!("================");

    for (server_name, server_tools) in &tools {
        println!("\n🔧 Server: {}", server_name);
        println!("   {} tools available", server_tools.len());

        for tool in server_tools {
            println!(
                "   • {} - {}",
                tool.name,
                tool.description.as_deref().unwrap_or("No description")
            );

            if let Some(schema) = &tool.input_schema {
                println!(
                    "     Input schema: {}",
                    serde_json::to_string_pretty(schema)
                        .unwrap_or_else(|_| "Invalid JSON".to_string())
                );
            }
        }
    }

    Ok(())
}

async fn list_resources(client: &MCPClient) -> Result<()> {
    let resources = client.list_all_resources().await?;

    if resources.is_empty() {
        println!("No resources available from connected servers.");
        return Ok(());
    }

    println!("Available Resources:");
    println!("===================");

    for (server_name, server_resources) in &resources {
        println!("\n📁 Server: {}", server_name);
        println!("   {} resources available", server_resources.len());

        for resource in server_resources {
            println!("   • {} ({})", resource.name, resource.uri);

            if let Some(desc) = &resource.description {
                println!("     {}", desc);
            }

            if let Some(mime_type) = &resource.mime_type {
                println!("     Type: {}", mime_type);
            }
        }
    }

    Ok(())
}

async fn call_tool(
    client: &MCPClient,
    server: &str,
    tool: &str,
    args: Option<&String>,
) -> Result<()> {
    let arguments = if let Some(args_str) = args {
        Some(serde_json::from_str(args_str)?)
    } else {
        None
    };

    println!(
        "Calling tool '{}' on server '{}' with args: {:?}",
        tool, server, arguments
    );

    match client.call_tool(server, tool, arguments).await {
        Ok(result) => {
            println!("Tool Result:");
            println!("============");

            if result.is_error {
                println!("❌ Error result:");
            } else {
                println!("✅ Success:");
            }

            for content in &result.content {
                println!("Type: {}", content.content_type);

                if let Some(text) = &content.text {
                    println!("Text: {}", text);
                }

                if let Some(data) = &content.data {
                    println!("Data: {}", serde_json::to_string_pretty(data)?);
                }

                println!("---");
            }
        }
        Err(e) => {
            println!("❌ Failed to call tool: {}", e);
        }
    }

    Ok(())
}

async fn read_resource(client: &MCPClient, server: &str, uri: &str) -> Result<()> {
    println!("Reading resource '{}' from server '{}'", uri, server);

    match client.read_resource(server, uri).await {
        Ok(content) => {
            println!("Resource Content:");
            println!("=================");
            println!("URI: {}", content.uri);

            if let Some(mime_type) = &content.mime_type {
                println!("MIME Type: {}", mime_type);
            }

            if let Some(text) = &content.text {
                println!("Text Content:\n{}", text);
            }

            if let Some(blob) = &content.blob {
                println!("Binary Content (base64): {} chars", blob.len());
                // Show first few characters of base64
                let preview = blob.chars().take(64).collect::<String>();
                println!("Preview: {}...", preview);
            }
        }
        Err(e) => {
            println!("❌ Failed to read resource: {}", e);
        }
    }

    Ok(())
}

async fn show_status(client: &MCPClient) -> Result<()> {
    let status = client.get_server_status();

    println!("Server Connection Status:");
    println!("========================");

    for (server_name, status) in &status {
        let status_icon = match status {
            ConnectionStatus::Connected => "✅",
            ConnectionStatus::Connecting => "🔄",
            ConnectionStatus::Disconnected => "⭕",
            ConnectionStatus::Failed(_) => "❌",
        };

        println!("{} {} - {:?}", status_icon, server_name, status);
    }

    Ok(())
}

async fn interactive_mode(mut client: MCPClient) -> Result<()> {
    println!("Interactive MCP Client");
    println!("Type 'help' for available commands, 'quit' to exit");

    loop {
        print!("mcp> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            "help" => {
                println!("Available commands:");
                println!("  status                    - Show server status");
                println!("  list-tools               - List all available tools");
                println!("  list-resources           - List all available resources");
                println!("  call <server> <tool> [args] - Call a tool");
                println!("  read <server> <uri>      - Read a resource");
                println!("  reconnect <server>       - Reconnect to a server");
                println!("  quit                     - Exit");
            }
            "status" => {
                show_status(&client).await?;
            }
            "list-tools" => {
                list_tools(&client).await?;
            }
            "list-resources" => {
                list_resources(&client).await?;
            }
            "call" => {
                if parts.len() < 3 {
                    println!("Usage: call <server> <tool> [args]");
                    continue;
                }

                let server = parts[1];
                let tool = parts[2];
                let args = if parts.len() > 3 {
                    Some(parts[3..].join(" "))
                } else {
                    None
                };

                call_tool(&client, server, tool, args.as_ref()).await?;
            }
            "read" => {
                if parts.len() < 3 {
                    println!("Usage: read <server> <uri>");
                    continue;
                }

                let server = parts[1];
                let uri = parts[2];

                read_resource(&client, server, uri).await?;
            }
            "reconnect" => {
                if parts.len() < 2 {
                    println!("Usage: reconnect <server>");
                    continue;
                }

                let server = parts[1];

                match client.reconnect_server(server).await {
                    Ok(()) => println!("✅ Reconnected to server: {}", server),
                    Err(e) => println!("❌ Failed to reconnect to server {}: {}", server, e),
                }
            }
            "quit" | "exit" => {
                println!("Shutting down...");
                client.shutdown().await?;
                break;
            }
            _ => {
                println!(
                    "Unknown command: {}. Type 'help' for available commands.",
                    command
                );
            }
        }
    }

    Ok(())
}

fn create_sample_config(path: &str) -> Result<()> {
    let config = MCPConfig::create_sample_config();
    let json = serde_json::to_string_pretty(&config)?;

    std::fs::write(path, json)?;

    println!("✅ Sample configuration created at: {}", path);
    println!("Edit the file to configure your MCP servers.");

    Ok(())
}
