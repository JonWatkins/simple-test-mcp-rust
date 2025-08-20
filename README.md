# Simple MCP Server Demo

A basic demonstration implementation of the Model Context Protocol (MCP) server in Rust. This is a simple example showing how to create an MCP server that provides tools and resources for AI agents.

## What is MCP?

The Model Context Protocol (MCP) is a standard protocol that enables AI agents to interact with external tools and resources in a structured way. It provides a JSON-RPC based interface for:

- Tool execution (functions that the AI can call)
- Resource management (files, data, etc.)
- Structured communication between AI agents and external services

## Demo Features

This simple demo MCP server provides:

### Tools
- **echo**: Echoes back the input message
- **add**: Adds two numbers together

### Resources
- **Example File**: A sample text file for demonstration

## Building and Running

### Prerequisites
- Rust (latest stable version)
- Cargo

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run
```

The server will start and wait for JSON-RPC requests on stdin/stdout.

## Testing the Demo

You can test the server by sending JSON-RPC requests to it. For example:

```bash
echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | cargo run
```

## Example Requests

### Initialize the server
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  }
}
```

### List available tools
```json
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "tools/list"
}
```

### Call a tool
```json
{
  "jsonrpc": "2.0",
  "id": "3",
  "method": "tools/call",
  "params": {
    "name": "echo",
    "arguments": {
      "message": "Hello, World!"
    }
  }
}
```

## Project Structure

```
test-mcp/
├── Cargo.toml                    # Rust dependencies and project configuration
├── src/
│   ├── main.rs                   # Main entry point
│   ├── server.rs                 # MCP server implementation
│   └── types.rs                  # Type definitions
└── README.md                     # This file
```

## Extending the Demo

To add new tools, modify the `tools` vector in the `McpServer::new()` method in `src/server.rs`:

```rust
Tool {
    name: "my_tool".to_string(),
    description: "Description of my tool".to_string(),
    input_schema: serde_json::json!({
        "type": "object",
        "properties": {
            "param1": {
                "type": "string",
                "description": "First parameter"
            }
        },
        "required": ["param1"]
    }),
}
```

Then implement the tool logic in the `execute_tool` method.

## Protocol Version

This demo implements MCP protocol version `2024-11-05`.
