# MCP Autocomplete Plugin: Complete Task List

---

## 🚀 Phase 1: Static Autocomplete (2 days)

### Rust Tasks

#### 1.1: Create Completion Types
- [ ] Define `CompletionRequest` struct (file_path, current_line, cursor_position, language)
- [ ] Define `CompletionResponse` struct (text, cursor_offset, confidence, metadata)
- [ ] Define `Position` struct (row, col)
- [ ] Add serde derives for JSON serialization

#### 1.2: Pattern Detection Engine
- [ ] Create `Pattern` enum (FunctionStart, IfStatement, ForLoop, etc.)
- [ ] Implement `detect_pattern()` function with regex patterns
- [ ] Create pattern → template mapping for common boilerplate

#### 1.3: Simple Completion Engine
- [ ] Create `CompletionEngine` struct
- [ ] Implement `get_completion()` method
- [ ] Add basic error handling
- [ ] Add logging (tracing)

#### 1.4: JSON-RPC Server Mode
- [ ] Add `--json-rpc` flag to CLI
- [ ] Create JSON-RPC request/response types
- [ ] Implement stdio read/write loop
- [ ] Add request routing (method name → handler)
- [ ] Handle `get_completion` method
- [ ] Add error responses for invalid requests

#### 1.5: Integration with Existing MCP Client
- [ ] Pass `MCPClient` instance to `CompletionEngine`
- [ ] Add initialization in `main.rs`
- [ ] Ensure JSON-RPC mode doesn't conflict with CLI mode

### Neovim/Lua Tasks

#### 1.6: Basic Plugin Structure
- [ ] Create `lua/mcp/init.lua` (main entry point)
- [ ] Create `lua/mcp/config.lua` (configuration)
- [ ] Create `lua/mcp/client.lua` (Rust process manager)
- [ ] Create module export pattern

#### 1.7: Rust Process Manager
- [ ] Implement `RustClient.new()` - spawn Rust binary with `--json-rpc`
- [ ] Set up stdin/stdout/stderr pipes
- [ ] Implement `RustClient:request()` - send JSON-RPC requests
- [ ] Implement response handler with `vim.schedule()` for async bridge
- [ ] Implement `RustClient:shutdown()`
- [ ] Add error handling for process crashes

#### 1.8: Ghost Text Module
- [ ] Create `lua/mcp/ghost_text.lua`
- [ ] Create namespace for virtual text
- [ ] Implement `show()` - display ghost text at cursor
- [ ] Implement `clear()` - remove ghost text
- [ ] Implement `accept()` - insert ghost text into buffer
- [ ] Handle multi-line suggestions

#### 1.9: Completion Request Module
- [ ] Create `lua/mcp/completion.lua`
- [ ] Implement `request_completion()` - gather buffer info and call Rust
- [ ] Add error handling for edge cases

#### 1.10: Basic Keymaps
- [ ] Create `lua/mcp/keymaps.lua`
- [ ] Implement Tab to accept suggestion
- [ ] Implement Esc to dismiss
- [ ] Make keymaps configurable

#### 1.11: Setup Function
- [ ] Implement `setup()` in `init.lua`
- [ ] Merge user config with defaults
- [ ] Start Rust client
- [ ] Register keymaps

### Testing

#### 1.12: Manual Testing
- [ ] Test: Type `function myFunc` → see ghost text
- [ ] Test: Press Tab → text inserted correctly
- [ ] Test: Press Esc → ghost text disappears
- [ ] Test: Works in Python, JavaScript, Rust files
- [ ] Test: Rust process starts/stops cleanly

#### 1.13: Error Handling Tests
- [ ] Test: Rust binary not found
- [ ] Test: Rust process crashes mid-session
- [ ] Test: Invalid completion request

---

## 🚀 Phase 2: LLM-Powered Autocomplete (3 days)

### Rust Tasks

#### 2.1: LLM Provider Trait
- [ ] Create `src/llm.rs` module
- [ ] Define `LLMProvider` trait with `complete()` method
- [ ] Define `LLMOptions` struct (temperature, top_p, stop sequences, max_tokens)

#### 2.2: Ollama Provider Implementation
- [ ] Add `reqwest` dependency to Cargo.toml
- [ ] Create `OllamaProvider` struct (base_url, model_name, options)
- [ ] Implement HTTP client for Ollama API (POST to `/api/generate`)
- [ ] Add timeout handling (5 seconds max)
- [ ] Add retry logic (1 retry on network error)
- [ ] Add error types for LLM failures

#### 2.3: Alternative Provider (Optional Backup)
- [ ] Create `OpenAIProvider` or `AnthropicProvider` struct
- [ ] Implement same `LLMProvider` trait
- [ ] Add API key handling from env vars

#### 2.4: Prompt Engineering
- [ ] Create `PromptBuilder` struct
- [ ] Implement `build_completion_prompt()` method
- [ ] Add language-specific prompt templates
- [ ] Add few-shot examples (optional)

#### 2.5: Response Cleaning
- [ ] Implement `clean_llm_response()` function
- [ ] Strip markdown code blocks
- [ ] Trim whitespace and remove duplicates
- [ ] Handle incomplete responses gracefully

#### 2.6: Update Completion Engine
- [ ] Add `llm_provider` field to `CompletionEngine`
- [ ] Update `get_completion()` to use LLM
- [ ] Keep static pattern detection as fallback
- [ ] Add "fast mode" toggle (static vs LLM)

#### 2.7: Configuration
- [ ] Add LLM config to `MCPConfig` (provider type, model name, base URL, API key)
- [ ] Add config validation
- [ ] Add sensible defaults

#### 2.8: Performance Monitoring
- [ ] Add timing metrics (prompt generation, LLM call, response cleaning)
- [ ] Log metrics with tracing
- [ ] Add performance warnings (>1s responses)

### Neovim/Lua Tasks

#### 2.9: Configuration Updates
- [ ] Add LLM settings to config (provider, model, timeout)
- [ ] Expose in `setup()` options
- [ ] Add config validation in Lua

#### 2.10: Loading Indicator
- [ ] Add visual feedback while waiting
- [ ] Statusline integration
- [ ] Clear indicator when response arrives

#### 2.11: Response Handling Updates
- [ ] Handle longer response times
- [ ] Add timeout on Lua side
- [ ] Show helpful error messages

### Testing

#### 2.12: LLM Setup Testing
- [ ] Install Ollama locally
- [ ] Download code model (`ollama pull codellama:7b`)
- [ ] Verify Ollama API works
- [ ] Test Rust can connect to Ollama

#### 2.13: Quality Testing
- [ ] Test: Python function completion quality
- [ ] Test: JavaScript object completion
- [ ] Test: Rust struct implementation
- [ ] Test: Comments → code generation
- [ ] Compare: Static vs LLM quality difference

#### 2.14: Performance Testing
- [ ] Measure: Average completion latency
- [ ] Test: Multiple rapid requests
- [ ] Test: Large file context
- [ ] Test: Edge cases (empty file, start of file)

#### 2.15: Error Recovery Testing
- [ ] Test: Ollama not running
- [ ] Test: Invalid model name
- [ ] Test: Network timeout
- [ ] Test: Malformed LLM response
- [ ] Test: Fallback to static patterns works

---

## 🚀 Phase 3: Neovim Integration Polish (3 days)

### Neovim/Lua Tasks

#### 3.1: Debouncing System
- [ ] Implement debounce timer in completion module
- [ ] Make delay configurable (default 300ms)
- [ ] Cancel timer on new keystroke
- [ ] Only send request after quiet period

#### 3.2: Request Cancellation
- [ ] Track pending request ID
- [ ] Implement cancel method in RustClient
- [ ] Cancel old request when new one starts
- [ ] Update Rust to handle cancellation

#### 3.3: Autocommand Setup
- [ ] Create `lua/mcp/autocmds.lua`
- [ ] Create augroup for MCP events
- [ ] Add `TextChangedI` autocmd (trigger debounced completion)
- [ ] Add `InsertLeave` autocmd (clear ghost text)
- [ ] Add `BufLeave` autocmd (clear ghost text)
- [ ] Add `VimLeavePre` autocmd (shutdown Rust client)

#### 3.4: Multi-line Ghost Text
- [ ] Detect multi-line completions
- [ ] Use `virt_lines` for additional lines
- [ ] Preserve indentation
- [ ] Handle line wrapping

#### 3.5: Advanced Keybindings
- [ ] Implement accept word (`<C-]>`)
- [ ] Implement accept line (`<C-[>`)
- [ ] Implement cycle suggestions (`<C-n>`/`<C-p>`)
- [ ] Make all keybindings configurable

#### 3.6: Visual Feedback Improvements
- [ ] Add confidence indicator in statusline
- [ ] Add source indicator ([MCP: LLM] or [MCP: Static])
- [ ] Add subtle fade-in animation
- [ ] Add custom highlight group

#### 3.7: Commands Module
- [ ] Create `lua/mcp/commands.lua`
- [ ] Implement `:MCPStatus` (show connection, LLM provider, performance, errors)
- [ ] Implement `:MCPToggle` (enable/disable completions)
- [ ] Implement `:MCPRestart` (restart Rust client)
- [ ] Implement `:MCPLogs` (open split with Rust stderr)

#### 3.8: Status Line Integration
- [ ] Create statusline component
- [ ] Show current state (Ready, Thinking, Error)
- [ ] Show server count
- [ ] Make it work with popular statusline plugins

#### 3.9: Error Handling UI
- [ ] Graceful error messages with `vim.notify()`
- [ ] Don't spam errors (debounce error messages)
- [ ] Show actionable suggestions
- [ ] Add error log buffer

#### 3.10: Configuration Validation
- [ ] Validate config on setup
- [ ] Check binary exists and is executable
- [ ] Warn about missing optional features
- [ ] Provide helpful error messages

### Rust Tasks

#### 3.11: Multiple Suggestions Support
- [ ] Modify `get_completion()` to return Vec<Completion>
- [ ] Generate 2-3 alternatives when requested
- [ ] Rank by confidence
- [ ] Add metadata (reasoning, source)

#### 3.12: Request Cancellation Support
- [ ] Add request ID tracking
- [ ] Implement `cancel_request` JSON-RPC method
- [ ] Stop in-flight LLM requests
- [ ] Clean up resources

#### 3.13: Streaming Responses (Optional)
- [ ] Implement streaming completion
- [ ] Send partial results as they arrive
- [ ] Update ghost text in real-time
- [ ] Handle cancellation mid-stream

#### 3.14: Performance Optimizations
- [ ] Cache LLM responses (simple LRU)
- [ ] Deduplicate identical requests
- [ ] Add request coalescing
- [ ] Profile and optimize hot paths

### Testing

#### 3.15: UX Testing
- [ ] Test: Debouncing feels natural
- [ ] Test: Cancellation works smoothly
- [ ] Test: Multi-line ghost text readable
- [ ] Test: Keybindings work as expected
- [ ] Test: Statusline updates correctly

#### 3.16: Edge Case Testing
- [ ] Test: Very long suggestions (100+ lines)
- [ ] Test: Rapid buffer switching
- [ ] Test: Leaving insert mode mid-request
- [ ] Test: Closing Neovim with pending requests
- [ ] Test: Extremely rapid typing

#### 3.17: Integration Testing
- [ ] Test: Works with other completion plugins (nvim-cmp)
- [ ] Test: Works with LSP
- [ ] Test: Works with snippet plugins
- [ ] Test: Doesn't conflict with other keybindings

#### 3.18: User Testing
- [ ] Get feedback from real users
- [ ] Identify pain points
- [ ] Test on different systems (macOS, Linux, Windows)
- [ ] Test with different terminals

---

## 🚀 Phase 4: Basic Context (2 days)

### Rust Tasks

#### 4.1: Context Types
- [ ] Create `src/context.rs` module
- [ ] Define `ContextSource` struct (name, content, relevance score)
- [ ] Define `ContextBundle` struct (collection of sources, total size tracking)
- [ ] Add serialization for passing to LLM

#### 4.2: File Content Reader
- [ ] Implement async file reading
- [ ] Add error handling for missing files
- [ ] Handle different encodings
- [ ] Add file size limits

#### 4.3: Context Gathering Logic
- [ ] Create `ContextGatherer` struct
- [ ] Implement `gather_context()` method
- [ ] Read current file
- [ ] Process open buffers (from Neovim)
- [ ] Return `ContextBundle`

#### 4.4: Context Ranking
- [ ] Implement simple relevance scoring
- [ ] Sort sources by relevance
- [ ] Prioritize smaller files

#### 4.5: Context Truncation
- [ ] Implement `truncate_to_size()` method (50KB default)
- [ ] Remove lowest relevance sources first
- [ ] Preserve current file always
- [ ] Smart truncation (keep complete functions)

#### 4.6: Enhanced Prompt Building
- [ ] Update `PromptBuilder` to use context
- [ ] Format context sources clearly
- [ ] Add source attribution in prompt
- [ ] Keep prompt under token limit

#### 4.7: Update Completion Engine
- [ ] Add `context_gatherer` to `CompletionEngine`
- [ ] Update `get_completion()` flow (gather context → build prompt → call LLM)
- [ ] Add timing for context gathering

#### 4.8: Configuration
- [ ] Add context settings to config
- [ ] Make it configurable per-language

### Neovim/Lua Tasks

#### 4.9: Buffer Collection
- [ ] Create `lua/mcp/buffers.lua` module
- [ ] Implement `get_open_buffers()` function
- [ ] Filter out unnamed/scratch buffers
- [ ] Add size limit

#### 4.10: Update Completion Request
- [ ] Modify `request_completion()` to include buffers
- [ ] Add buffer info to request payload
- [ ] Handle large buffer sets efficiently
- [ ] Make buffer collection optional

#### 4.11: Context Preview Command
- [ ] Create `:MCPContext` command
- [ ] Show what context is being used
- [ ] Display in floating window
- [ ] Allow user to toggle sources

#### 4.12: Configuration
- [ ] Add context settings to Lua config
- [ ] Validate settings

### Testing

#### 4.13: Context Quality Testing
- [ ] Test: Suggestions reference earlier function
- [ ] Test: Suggestions use imported types
- [ ] Test: Suggestions consistent with file style
- [ ] Test: Suggestions use variables from other buffers

#### 4.14: Performance Testing
- [ ] Test: Context gathering latency
- [ ] Test: Works with 10+ open buffers
- [ ] Test: Works with large files (10,000 lines)
- [ ] Test: Doesn't slow down typing

#### 4.15: Edge Case Testing
- [ ] Test: Binary files in buffers
- [ ] Test: Very large buffers
- [ ] Test: Unnamed buffers
- [ ] Test: Files with special characters
- [ ] Test: Empty files

---

## 🚀 Phase 5: MCP-Powered Context (5 days)

### Rust Tasks

#### 5.1: MCP Integration Planning
- [ ] Review existing `MCPClient` capabilities
- [ ] Identify useful MCP operations
- [ ] Plan MCP server usage (filesystem, github, fetch)

#### 5.2: Related Files Discovery
- [ ] Implement `find_related_files()` method using filesystem MCP
- [ ] Find files in same directory
- [ ] Find files with similar names
- [ ] Find imported/referenced files
- [ ] Add language-specific heuristics
- [ ] Rank by relevance

#### 5.3: MCP File Reading
- [ ] Implement `read_file_via_mcp()` method
- [ ] Use `read_resource` from MCP server
- [ ] Handle different content types
- [ ] Add caching to avoid repeated reads
- [ ] Add error handling for missing files

#### 5.4: Git Context Integration
- [ ] Implement `get_git_context()` method using github MCP
- [ ] Get recent commits for current file
- [ ] Get PR context (if in PR)
- [ ] Format git context for prompt
- [ ] Make it optional (graceful degradation)

#### 5.5: Documentation Context
- [ ] Implement `get_documentation()` method using fetch MCP
- [ ] Detect libraries being used
- [ ] Fetch relevant docs
- [ ] Add doc caching
- [ ] Make it optional

#### 5.6: Context Source Prioritization
- [ ] Implement intelligent prioritization
- [ ] Add configurable weights
- [ ] Balance context size vs relevance

#### 5.7: Graceful Degradation
- [ ] Handle MCP server unavailable
- [ ] Handle MCP operation timeout
- [ ] Fall back to simpler context
- [ ] Log warnings, not errors
- [ ] Track which servers are working

#### 5.8: Context Caching
- [ ] Implement `ContextCache` struct
- [ ] Cache file contents (TTL: 30 seconds)
- [ ] Cache resource listings (TTL: 60 seconds)
- [ ] Cache documentation (TTL: 5 minutes)
- [ ] Add cache invalidation on file save

#### 5.9: Update Context Gatherer
- [ ] Add MCP client to `ContextGatherer`
- [ ] Update `gather_context()` to use MCP
- [ ] Orchestrate multiple MCP operations
- [ ] Run operations in parallel (tokio::join!)
- [ ] Combine all context sources

#### 5.10: Performance Optimization
- [ ] Add request timeouts (500ms max for context)
- [ ] Add request prioritization
- [ ] Add circuit breaker pattern

### Neovim/Lua Tasks

#### 5.11: MCP Server Status Display
- [ ] Update `:MCPStatus` to show MCP servers
- [ ] Add visual indicators (✓/✗/⚠)

#### 5.12: Context Source Configuration
- [ ] Add MCP-specific config options
- [ ] Expose in `setup()` options

#### 5.13: Context Preview Updates
- [ ] Update `:MCPContext` to show MCP sources
- [ ] Show in tree/hierarchical view

### Testing

#### 5.14: MCP Integration Testing
- [ ] Test: Filesystem MCP finds related files
- [ ] Test: Can read files via MCP
- [ ] Test: Works when MCP server offline
- [ ] Test: Handles slow MCP responses

#### 5.15: Context Quality Testing
- [ ] Test: Suggestions use types from imported files
- [ ] Test: Suggestions consistent with git history
- [ ] Test: Suggestions follow documentation patterns
- [ ] Compare: MCP context vs file-only context quality

#### 5.16: Performance Testing
- [ ] Test: Context gathering with MCP <500ms
- [ ] Test: Doesn't block on slow MCP servers
- [ ] Test: Cache hit rate >50%
- [ ] Test: Circuit breaker activates correctly

#### 5.17: Multi-Server Testing
- [ ] Test: Works with all servers connected
- [ ] Test: Works with some servers offline
- [ ] Test: Handles server reconnection
- [ ] Test: Load balances across servers

---

## 🚀 Phase 6: RAG & Vector Search (1-2 weeks)

### Research & Setup

#### 6.1: Vector Store Research
- [ ] Evaluate vector store options (Tantivy, Qdrant, in-memory, SQLite)
- [ ] Choose based on performance, memory usage, ease of integration
- [ ] Document decision

#### 6.2: Embedding Model Selection
- [ ] Research embedding models (all-MiniLM-L6-v2, CodeBERT, StarEncoder)
- [ ] Evaluate model format (ONNX Runtime, Torch, REST API)
- [ ] Choose model
- [ ] Test inference speed

### Rust Tasks

#### 6.3: Vector Store Implementation
- [ ] Create `src/vector_store.rs` module
- [ ] Define `VectorStore` trait (insert, search, delete, clear)
- [ ] Implement chosen vector store
- [ ] Add persistence (save/load index)

#### 6.4: Embedding Model Integration
- [ ] Create `src/embeddings.rs` module
- [ ] Define `EmbeddingModel` trait (embed, embed_batch)
- [ ] Implement model wrapper
- [ ] Add error handling
- [ ] Add caching

#### 6.5: Code Chunking Strategy
- [ ] Implement `chunk_code()` function
- [ ] Language-aware chunking (by function/class/impl)
- [ ] Overlapping windows for context
- [ ] Preserve complete semantic units
- [ ] Add metadata to chunks

#### 6.6: RAG Engine Core
- [ ] Create `src/rag.rs` module
- [ ] Define `RAGEngine` struct
- [ ] Implement `new()` constructor

#### 6.7: Semantic Search
- [ ] Implement `find_relevant_context()` method
- [ ] Generate query embedding
- [ ] Search vector store
- [ ] Re-rank results
- [ ] Return top-k snippets
- [ ] Add filtering and score thresholds

#### 6.8: Codebase Indexing
- [ ] Implement `index_codebase()` method
- [ ] Get files from MCP filesystem server
- [ ] Filter by file type
- [ ] Read, chunk, embed, and insert
- [ ] Show progress
- [ ] Add resume capability

#### 6.9: Incremental Indexing
- [ ] Implement `index_file()` method
- [ ] Implement `delete_file_chunks()` method
- [ ] Track file modification times
- [ ] Only reindex if changed

#### 6.10: Background Indexing
- [ ] Create background indexing task
- [ ] Use separate tokio task
- [ ] Don't block completions
- [ ] Add priority queue
- [ ] Add rate limiting

#### 6.11: RAG Context Integration
- [ ] Update `ContextGatherer` to use RAG
- [ ] Add `find_semantic_context()` method
- [ ] Integrate with existing context sources
- [ ] Balance RAG vs MCP context

#### 6.12: Index Management
- [ ] Add `:index_status` JSON-RPC method
- [ ] Add `:rebuild_index` method
- [ ] Add `:index_file` method
- [ ] Add index versioning

### Neovim/Lua Tasks

#### 6.13: Indexing Commands
- [ ] Implement `:MCPIndex` command
- [ ] Implement `:MCPIndexStatus` command
- [ ] Implement `:MCPRebuildIndex` command

#### 6.14: Auto-indexing
- [ ] Add autocommand for `BufWritePost` (index saved file)
- [ ] Add autocommand for `VimEnter` (start background indexing)
- [ ] Make auto-indexing configurable

#### 6.15: Progress Indicators
- [ ] Show indexing progress in statusline
- [ ] Show notification when indexing complete
- [ ] Don't spam notifications

#### 6.16: Configuration
- [ ] Add RAG settings to config
- [ ] Validate configuration

### Testing

#### 6.17: Embedding Quality Testing
- [ ] Test: Similar code has similar embeddings
- [ ] Test: Different code has different embeddings
- [ ] Measure: Embedding generation speed
- [ ] Test: Batch embedding faster than individual

#### 6.18: Search Quality Testing
- [ ] Test: Search finds semantically similar code
- [ ] Test: Search ranks by relevance
- [ ] Test: Search filters work correctly
- [ ] Compare: Keyword search vs semantic search

#### 6.19: Indexing Performance Testing
- [ ] Test: Index 1000 files <5 minutes
- [ ] Test: Incremental update <100ms
- [ ] Test: Background indexing doesn't block UI
- [ ] Measure: Memory usage during indexing

#### 6.20: End-to-End RAG Testing
- [ ] Test: Completions use similar code from codebase
- [ ] Test: Completions learn from existing patterns
- [ ] Test: Works across multiple languages
- [ ] Compare: Completion quality with vs without RAG

#### 6.21: Scale Testing
- [ ] Test: 10,000 files indexed
- [ ] Test: 100,000+ code chunks searchable
- [ ] Test: Search performance at scale (<100ms)
- [ ] Test: Memory usage stays reasonable (<500MB)
- [ ] Test: Index persistence (save/load works)

#### 6.22: Edge Case Testing
- [ ] Test: Duplicate code detection
- [ ] Test: Empty files
- [ ] Test: Files with minimal code
- [ ] Test: Generated/minified code
- [ ] Test: Mixed languages in same file
- [ ] Test: Very large single files (10k+ lines)

#### 6.23: Integration Testing
- [ ] Test: RAG + MCP context together
- [ ] Test: RAG + open buffers + file context
- [ ] Test: Prioritization works correctly
- [ ] Test: Context size limits respected
- [ ] Test: All context sources contribute

---

## 🔧 Cross-Phase Tasks (Ongoing)

### Code Quality

#### Error Handling
- [ ] Audit all error types
- [ ] Ensure proper error propagation
- [ ] Add context to errors
- [ ] Create custom error types
- [ ] Document error handling patterns

#### Logging & Tracing
- [ ] Add structured logging with tracing
- [ ] Add spans for major operations
- [ ] Add timing information
- [ ] Configure log levels properly
- [ ] Ensure no sensitive data in logs

#### Configuration Validation
- [ ] Validate all config values
- [ ] Provide helpful error messages
- [ ] Add config migration (if format changes)
- [ ] Document all config options

#### Code Documentation
- [ ] Add rustdoc comments to public APIs
- [ ] Add module-level documentation
- [ ] Add examples in documentation
- [ ] Document complex algorithms

#### Unit Tests
- [ ] Test pattern detection
- [ ] Test prompt building
- [ ] Test context gathering
- [ ] Test MCP integration
- [ ] Test chunking logic
- [ ] Test vector search
- [ ] Aim for >80% code coverage

#### Integration Tests
- [ ] Test JSON-RPC protocol
- [ ] Test Rust ↔ Neovim communication
- [ ] Test complete completion flow
- [ ] Test error recovery
- [ ] Test concurrent requests

### Performance

#### Profiling
- [ ] Profile completion latency
- [ ] Profile memory usage
- [ ] Profile CPU usage
- [ ] Identify bottlenecks
- [ ] Use flamegraphs for visualization

#### Optimization
- [ ] Optimize hot paths
- [ ] Reduce allocations
- [ ] Use async efficiently
- [ ] Add connection pooling
- [ ] Optimize prompt building

#### Caching Strategy
- [ ] Document what gets cached
- [ ] Document cache invalidation rules
- [ ] Implement cache size limits
- [ ] Add cache hit/miss metrics
- [ ] Add cache clear command

#### Resource Management
- [ ] Ensure proper cleanup on shutdown
- [ ] Handle SIGTERM/SIGINT gracefully
- [ ] Close file handles properly
- [ ] Stop background tasks cleanly
- [ ] Prevent resource leaks

### Neovim/Lua Quality

#### Lua Code Quality
- [ ] Add type annotations (EmmyLua/LuaLS)
- [ ] Consistent code style
- [ ] Modular organization
- [ ] No global variables (except vim)
- [ ] Document all public functions

#### Lua Error Handling
- [ ] Protect all vim.api calls with pcall
- [ ] Handle missing dependencies
- [ ] Provide helpful error messages
- [ ] Don't crash Neovim on errors
- [ ] Log errors properly

#### Lua Testing
- [ ] Set up testing framework (plenary.nvim)
- [ ] Test configuration parsing
- [ ] Test keybinding setup
- [ ] Test autocommand setup
- [ ] Test ghost text display

### Security

#### Input Validation
- [ ] Validate all JSON-RPC inputs
- [ ] Sanitize file paths (no path traversal)
- [ ] Validate buffer contents size
- [ ] Rate limit requests
- [ ] Prevent command injection

#### Secrets Management
- [ ] Never log API keys
- [ ] Use environment variables for secrets
- [ ] Don't commit secrets to git
- [ ] Document secure configuration
- [ ] Warn about insecure configurations

#### Dependency Security
- [ ] Audit dependencies regularly
- [ ] Update dependencies for security patches
- [ ] Use `cargo audit`
- [ ] Pin dependency versions
- [ ] Document security considerations

### Compatibility

#### Platform Compatibility
- [ ] Test on Linux (Ubuntu, Arch)
- [ ] Test on macOS (Intel, M1/M2)
- [ ] Test on Windows (if supporting)
- [ ] Handle path separators correctly
- [ ] Handle line endings correctly

#### Neovim Version Compatibility
- [ ] Test on Neovim 0.9+
- [ ] Test on Neovim 0.10+
- [ ] Document minimum version
- [ ] Handle API changes gracefully
- [ ] Feature detection

#### Terminal Compatibility
- [ ] Test in various terminals (iTerm2, Alacritty, kitty, wezterm, etc.)
- [ ] Test color rendering
- [ ] Test special characters

---

## 📦 Build & Distribution

### Build

#### Build Configuration
- [ ] Optimize release builds (opt-level = 3, lto = true)
- [ ] Strip debug symbols
- [ ] Add build profiles

#### Cross-Compilation
- [ ] Build Linux x86_64 binary
- [ ] Build macOS x86_64 binary
- [ ] Build macOS ARM64 binary
- [ ] Build Windows binary (if supporting)
- [ ] Test binaries on target platforms

### Installation

#### Installation Methods
- [ ] Document manual installation
- [ ] Create install script (curl | sh)
- [ ] Create homebrew formula (macOS/Linux)
- [ ] Create cargo install instructions

#### Neovim Plugin Manager Support
- [ ] Support lazy.nvim (write example config, document build steps)
- [ ] Support packer.nvim (write example config)
- [ ] Support vim-plug (write example config)

#### Dependency Management
- [ ] Document runtime dependencies (Ollama, MCP servers)
- [ ] Check for dependencies on startup
- [ ] Provide helpful error messages if missing
- [ ] Document how to install dependencies

---

## 📚 Documentation

### User Documentation

#### README
- [ ] Write compelling README with value proposition
- [ ] Add screenshots/GIFs
- [ ] Add feature list
- [ ] Add quick start guide
- [ ] Add installation instructions
- [ ] Add badges (CI, version, license)

#### Installation Guide
- [ ] Write detailed installation steps
- [ ] Cover all supported platforms
- [ ] Document troubleshooting
- [ ] Include common issues

#### Configuration Guide
- [ ] Document all config options
- [ ] Provide example configurations
- [ ] Explain each option's purpose
- [ ] Show default values
- [ ] Add configuration recipes

#### User Guide
- [ ] Write getting started tutorial
- [ ] Document basic usage
- [ ] Document advanced features
- [ ] Add tips and tricks
- [ ] Add FAQ section

#### Command Reference
- [ ] Document all user commands
- [ ] Document all keybindings
- [ ] Add examples for each

#### Troubleshooting Guide
- [ ] Document common issues
- [ ] Explain error messages
- [ ] Add debugging steps
- [ ] Document known limitations
- [ ] Provide workarounds

### Developer Documentation

#### Architecture Documentation
- [ ] Write architecture overview
- [ ] Create architecture diagrams
- [ ] Document component interactions
- [ ] Explain design decisions
- [ ] Document extension points

#### Contributing Guide
- [ ] Write CONTRIBUTING.md
- [ ] Explain development setup
- [ ] Document code style
- [ ] Explain PR process
- [ ] Add testing guidelines

#### API Documentation
- [ ] Generate rustdoc documentation
- [ ] Host documentation online (docs.rs)
- [ ] Document JSON-RPC protocol
- [ ] Document Lua API
- [ ] Add examples for each API

#### Development Guide
- [ ] Document how to build from source
- [ ] Explain project structure
- [ ] Document development workflow
