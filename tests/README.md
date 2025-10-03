# JSON-RPC Server Integration Tests

This directory contains integration tests for the Rust JSON-RPC server that validates all the functionality Neovim needs.

## Setup

```bash
# Install Python test dependencies
pip install -r requirements.txt
```

## Running Tests

```bash
# Run all tests
pytest tests/test_jsonrpc_server.py -v

# Run specific test class
pytest tests/test_jsonrpc_server.py::TestCompletionEndpoint -v

# Run with detailed output
pytest tests/test_jsonrpc_server.py -vv -s

# Run with coverage
pytest tests/test_jsonrpc_server.py --cov=. -v
```

## Test Coverage

The test suite validates:

### Protocol Compliance
- ✅ JSON-RPC 2.0 protocol compliance
- ✅ Invalid JSON handling
- ✅ Invalid request handling
- ✅ Method not found errors

### Core Functionality
- ✅ Status endpoint
- ✅ Completion endpoint with all parameters
- ✅ Pattern detection for Rust, Python, JavaScript/TypeScript
- ✅ No completion when no pattern detected
- ✅ Completion with surrounding context

### MCP Integration
- ✅ List tools
- ✅ List resources
- ✅ Call tool (when servers are configured)
- ✅ Read resource (when servers are configured)

### Performance
- ✅ Response time < 100ms for static completions
- ✅ Processing time < 50ms
- ✅ Multiple rapid requests handling

### Edge Cases
- ✅ Empty line completion
- ✅ Very long lines
- ✅ Unsupported languages
- ✅ Missing parameters

### Graceful Shutdown
- ✅ Shutdown command

## What Neovim Needs

Based on these tests, Neovim's Lua plugin needs to:

1. **Start the server**: Spawn `rusty --json-rpc --config <path>` with stdin/stdout pipes
2. **Send requests**: JSON-RPC 2.0 format over stdin
3. **Read responses**: One JSON object per line from stdout
4. **Handle errors**: Check for `error` field in responses
5. **Primary method**: `get_completion` with:
   - `file_path`, `language`, `current_line`, `cursor_position`
   - Optional: `context_before`, `context_after` arrays

## Example Request/Response

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "get_completion",
  "params": {
    "file_path": "test.rs",
    "language": "rust",
    "current_line": "fn main",
    "cursor_position": {"line": 0, "column": 7},
    "context_before": [],
    "context_after": []
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "completions": [
      {
        "text": "() {\n    \n}",
        "cursor_offset": -2,
        "confidence": 0.8,
        "source": "static",
        "metadata": {"pattern": "FunctionStart"}
      }
    ],
    "processing_time_ms": 5
  }
}
```
