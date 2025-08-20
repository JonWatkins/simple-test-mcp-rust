mod server;
mod types;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tracing::{error, info, warn};

use crate::server::McpServer;
use crate::types::{JsonRpcRequest, McpError, McpResponse};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr instead of stdout to avoid interfering with JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    info!("Starting MCP server...");

    let server = McpServer::new();

    // For simplicity, we'll use stdin/stdout for communication
    // In a real implementation, you might want to use TCP or other transport
    let stdin = tokio::io::stdin();
    let mut stdin = tokio::io::BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    info!("MCP server ready. Waiting for requests...");

    let mut line = String::new();

    while stdin.read_line(&mut line).await? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(request) => {
                let request_id = request.id.clone();
                match server.handle_request(request).await {
                    Ok(Some(response)) => {
                        let response_json = serde_json::to_string(&response)?;
                        stdout.write_all(response_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                    Ok(None) => {
                        // No response needed for notifications
                    }
                    Err(e) => {
                        error!("Error handling request: {}", e);
                        let error_response = McpResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request_id
                                .unwrap_or_else(|| serde_json::Value::String("error".to_string())),
                            result: None,
                            error: Some(McpError {
                                code: -32603,
                                message: format!("Internal error: {}", e),
                            }),
                        };
                        let error_json = serde_json::to_string(&error_response)?;
                        stdout.write_all(error_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse request: {}", e);
                let error_response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::String("parse_error".to_string()),
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let error_json = serde_json::to_string(&error_response)?;
                stdout.write_all(error_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        line.clear();
    }

    Ok(())
}
