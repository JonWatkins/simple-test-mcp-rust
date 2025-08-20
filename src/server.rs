use anyhow::Result;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::types::*;

pub struct McpServer {
    tools: Vec<Tool>,
    resources: Vec<Resource>,
    prompts: Vec<Prompt>,
}

impl McpServer {
    pub fn new() -> Self {
        let tools = vec![
            Tool {
                name: "echo".to_string(),
                description: "Echoes back the input message".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to echo"
                        }
                    },
                    "required": ["message"]
                }),
            },
            Tool {
                name: "add".to_string(),
                description: "Adds two numbers together".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "a": {
                            "type": "number",
                            "description": "First number"
                        },
                        "b": {
                            "type": "number",
                            "description": "Second number"
                        }
                    },
                    "required": ["a", "b"]
                }),
            },
        ];

        let resources = vec![Resource {
            uri: "file:///example.txt".to_string(),
            name: "Example File".to_string(),
            description: "An example text file".to_string(),
            mime_type: "text/plain".to_string(),
        }];

        let prompts = vec![Prompt {
            name: "hello".to_string(),
            description: "Returns a friendly greeting".to_string(),
        }];

        Self {
            tools,
            resources,
            prompts,
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,
            "prompts/list" => self.handle_prompts_list(request).await,
            "prompts/get" => self.handle_prompts_get(request).await,
            "initialized" => self.handle_initialized().await,
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        }
    }

    async fn handle_initialize(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        let params: InitializeParams =
            serde_json::from_value(request.params.unwrap_or_else(|| serde_json::json!({})))?;
        info!(
            "Initializing MCP server with protocol version: {}",
            params.protocol_version
        );

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    },
                    "resources": {
                        "listChanged": true
                    },
                    "prompts": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "leap-mcp",
                    "version": "0.1.0"
                }
            })),
            error: None,
        }))
    }

    async fn handle_tools_list(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        info!("Listing tools");
        let tools_json: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                })
            })
            .collect();

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "tools": tools_json
            })),
            error: None,
        }))
    }

    async fn handle_tools_call(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        let params: ToolCallParams = serde_json::from_value(
            request
                .params
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?,
        )?;
        info!("Calling tool: {}", params.name);
        let result = self.execute_tool(&params.name, &params.arguments).await?;

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": result
                    }
                ],
                "isError": false
            })),
            error: None,
        }))
    }

    async fn handle_resources_list(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        info!("Listing resources");
        let resources_json: Vec<serde_json::Value> = self
            .resources
            .iter()
            .map(|resource| {
                serde_json::json!({
                    "uri": resource.uri,
                    "name": resource.name,
                    "description": resource.description,
                    "mimeType": resource.mime_type
                })
            })
            .collect();

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "resources": resources_json
            })),
            error: None,
        }))
    }

    async fn handle_resources_read(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        let params: ResourceReadParams = serde_json::from_value(
            request
                .params
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?,
        )?;
        info!("Reading resource: {}", params.uri);
        let content = self.read_resource(&params.uri).await?;

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "contents": [
                    {
                        "uri": params.uri,
                        "mimeType": "text/plain",
                        "text": content
                    }
                ]
            })),
            error: None,
        }))
    }

    async fn handle_prompts_list(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        info!("Listing prompts");
        let prompts_json: Vec<serde_json::Value> = self
            .prompts
            .iter()
            .map(|prompt| {
                serde_json::json!({
                    "name": prompt.name,
                    "description": prompt.description
                })
            })
            .collect();

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "prompts": prompts_json
            })),
            error: None,
        }))
    }

    async fn handle_prompts_get(&self, request: JsonRpcRequest) -> Result<Option<McpResponse>> {
        let params: PromptGetParams = serde_json::from_value(
            request
                .params
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?,
        )?;
        info!("Getting prompt: {}", params.name);

        let content_text = match params.name.as_str() {
            "hello" => "Hello from leap-mcp prompts!".to_string(),
            _ => return Err(anyhow::anyhow!("Unknown prompt: {}", params.name)),
        };

        Ok(Some(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.unwrap_or(serde_json::Value::Null),
            result: Some(serde_json::json!({
                "messages": [
                    { "role": "user", "content": [ { "type": "text", "text": content_text } ] }
                ]
            })),
            error: None,
        }))
    }

    async fn handle_initialized(&self) -> Result<Option<McpResponse>> {
        info!("Received initialized notification");
        // After client is initialized, notify that lists changed
        self.send_notification("tools/listChanged", serde_json::json!({}))
            .await?;
        self.send_notification("resources/listChanged", serde_json::json!({}))
            .await?;
        self.send_notification("prompts/listChanged", serde_json::json!({}))
            .await?;
        // No response for notifications
        Ok(None)
    }

    async fn execute_tool(
        &self,
        name: &str,
        arguments: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        match name {
            "echo" => {
                let message = arguments
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'message' argument"))?;
                Ok(format!("Echo: {}", message))
            }
            "add" => {
                let a = arguments
                    .get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'a' argument"))?;
                let b = arguments
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'b' argument"))?;
                Ok(format!("{} + {} = {}", a, b, a + b))
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    async fn read_resource(&self, uri: &str) -> Result<String> {
        match uri {
            "file:///example.txt" => Ok("This is an example text file content.\nIt contains some sample text for demonstration purposes.".to_string()),
            _ => Err(anyhow::anyhow!("Resource not found: {}", uri))
        }
    }

    async fn send_notification(&self, method: &str, params: serde_json::Value) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let mut stdout = tokio::io::stdout();
        let notification_json = serde_json::to_string(&notification)?;
        stdout.write_all(notification_json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;

        Ok(())
    }
}
