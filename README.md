# Rusty

[![CI](https://github.com/USERNAME/rusty/workflows/CI/badge.svg)](https://github.com/USERNAME/rusty/actions)
[![codecov](https://codecov.io/gh/USERNAME/rusty/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/rusty)

An MCP (Model Context Protocol) hub alternative written in Rust with Neovim autocomplete integration. 

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Neovim Lua Layer                         │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│ │   Autocomplete  │ │   RAG Context   │ │   Code Actions  │ │
│ │   Interface     │ │   Manager       │ │   Handler       │ │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    FFI/JSON-RPC Bridge                      │
├─────────────────────────────────────────────────────────────┤
│                    Rust MCP Client Core                     │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│ │   MCP Manager   │ │  Context Cache  │ │ Completion Engine││
│ │   (rmcp-based)  │ │  (RAG Store)    │ │  (LLM Interface)│ │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                   MCP Server Connections                    │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│ │  Filesystem     │ │      Git        │ │    Language     │ │
│ │    Server       │ │    Server       │ │     Server      │ │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```
