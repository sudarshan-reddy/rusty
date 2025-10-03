# Neovim User Experience Design for MCP-Powered Autocomplete

## Core Design Philosophy

**Goal**: Create a GitHub Copilot-like autocomplete experience that's:
- **Invisible when working**: Suggestions appear naturally, don't interrupt flow
- **Contextually intelligent**: Uses MCP servers to gather rich context (filesystem, git, docs)
- **Fast and responsive**: Sub-100ms latency for inline suggestions
- **Configurable**: Users control aggressiveness, sources, and behavior
- **Transparent**: Clear about what context is being used

## User Experience Flow

### 1. Initial Setup Experience

**First-time user workflow:**

```vim
:MCPInstall
" Shows setup wizard:
" ┌─────────────────────────────────────────┐
" │ MCP Autocomplete Setup                  │
" ├─────────────────────────────────────────┤
" │ [1/3] Configuration Found               │
" │ ✓ .vscode/mcp.json detected             │
" │ ✓ 3 servers configured                  │
" │                                         │
" │ Servers:                                │
" │  ✓ filesystem (local files)             │
" │  ✓ github (repository context)          │
" │  ✓ fetch (web documentation)            │
" │                                         │
" │ [Next] [Configure] [Skip]               │
" └─────────────────────────────────────────┘
```

**Configuration preferences:**

```vim
:MCPConfig
" Interactive configuration menu:
" ┌─────────────────────────────────────────┐
" │ MCP Autocomplete Preferences            │
" ├─────────────────────────────────────────┤
" │ Suggestion Behavior:                    │
" │  [x] Auto-show inline (Copilot style)   │
" │  [ ] Show on manual trigger only        │
" │  [ ] Show in completion menu            │
" │                                         │
" │ Context Sources (toggle with <Space>):  │
" │  [x] Current file                       │
" │  [x] Open buffers                       │
" │  [x] Project files (via filesystem MCP) │
" │  [x] Git history (via git MCP)          │
" │  [ ] Web docs (via fetch MCP)           │
" │                                         │
" │ Performance:                            │
" │  Trigger delay: [200]ms                 │
" │  Max context: [50]KB                    │
" │                                         │
" │ [Save] [Reset] [Cancel]                 │
" └─────────────────────────────────────────┘
```

### 2. Active Coding Experience

#### **Inline Ghost Text (Primary UX)**

When user is typing, ghost text appears automatically:

```python
def calculate_fibonacci(n: int) -> int:
    """Calculate the nth Fibonacci number."""
    if n <= 1:
        return n
    # User cursor is here: |
    # Ghost text appears in gray:
    return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)
```

**Visual design:**
- Ghost text in `#6B7280` gray (dark mode) or `#9CA3AF` (light mode)
- Inline, at cursor position
- Fades in smoothly (50ms animation)
- Shows context indicator in statusline: `[MCP: filesystem+github]`

**Keybindings (Copilot-inspired):**
- `<Tab>` - Accept entire suggestion
- `<C-]>` - Accept next word
- `<C-[>` - Accept next line
- `<Esc>` - Dismiss suggestion
- `<C-n>` - Cycle to next suggestion (if multiple available)
- `<C-p>` - Cycle to previous suggestion

#### **Context Awareness Indicators**

Status line shows what context is being used:

```
Normal mode:
  ┌──────────────────────────────────────────────────┐
  │ main.py [+] [MCP: 3 servers, 247KB context] │ 42:15
  └──────────────────────────────────────────────────┘

Insert mode with suggestion:
  ┌──────────────────────────────────────────────────┐
  │ main.py [+] [MCP: ✓ 87% confident] [github: ...] │
  └──────────────────────────────────────────────────┘
```

**Context preview popup** (triggered with `<Leader>mc`):

```
┌─────────────────────────────────────────┐
│ MCP Context Preview                     │
├─────────────────────────────────────────┤
│ Current Suggestion Context:             │
│                                         │
│ 📁 Filesystem (filesystem MCP):         │
│  • main.py (current file)               │
│  • utils/helpers.py (related)           │
│  • tests/test_main.py (tests)           │
│                                         │
│ 🔀 Git (github MCP):                    │
│  • Recent commits on this function      │
│  • PR #123 discussion                   │
│                                         │
│ 🧠 Vector Store:                        │
│  • 15 similar code patterns found       │
│  • Confidence: 87%                      │
│                                         │
│ Context size: 247KB / 50KB limit        │
│ Generation time: 89ms                   │
│                                         │
│ [Adjust Context] [Refresh]              │
└─────────────────────────────────────────┘
```

### 3. Completion Menu Integration (Alternative Mode)

For users who prefer traditional completion menus:

```vim
function getData(id: number) {
  const result = fetch█
                    │
                    ▼
┌────────────────────────────────────────────┐
│ fetch(url: string, options?: RequestInit)  │ [MCP: github]
│ fetchUserData(userId: number)              │ [MCP: filesystem]
│ fetchFromCache(key: string)                │ [Local]
│ fetchWithRetry(url: string, retries: 3)    │ [MCP: Vector]
└────────────────────────────────────────────┘
```

**Features:**
- MCP suggestions marked with `[MCP: source]`
- Context-aware ranking (MCP suggestions prioritized when relevant)
- Rich documentation preview from MCP resources
- Works with nvim-cmp, completion-nvim, or native completion

### 4. Multi-line Suggestions

For larger code blocks (triggered with `<Leader>mg`):

```python
def process_user_data(user_id):
    """Process user data from database."""
    # User triggers multi-line with <Leader>mg
    # Ghost overlay appears:
    
    ┌─────────────────────────────────────────┐
    │ [1/3] MCP Suggestion (Shift+Tab/Tab)    │
    ├─────────────────────────────────────────┤
    │ try:                                    │
    │     user = database.get_user(user_id)   │
    │     if not user:                        │
    │         raise ValueError("Not found")   │
    │     return {                            │
    │         "id": user.id,                  │
    │         "name": user.name,              │
    │         "email": user.email             │
    │     }                                   │
    │ except DatabaseError as e:              │
    │     logger.error(f"DB error: {e}")      │
    │     raise                               │
    │                                         │
    │ Context: database.py, user model, tests │
    │ [Accept: Tab] [Next: Shift+Tab] [X: Esc]│
    └─────────────────────────────────────────┘
```

### 5. Intelligent Context Commands

**Manual context exploration:**

```vim
:MCPContext
" Shows interactive context browser:
" ┌─────────────────────────────────────────┐
" │ MCP Context Explorer                    │
" ├─────────────────────────────────────────┤
" │ 📊 Current Context (for main.py:42)     │
" │                                         │
" │ 🎯 Relevant Files (filesystem):         │
" │  [x] utils/helpers.py         (12KB)    │
" │  [x] models/user.py          (8KB)      │
" │  [ ] services/auth.py        (15KB)     │
" │  [ ] config/settings.py      (3KB)      │
" │                                         │
" │ 📚 Documentation (fetch):               │
" │  [x] Python requests docs               │
" │  [ ] FastAPI best practices             │
" │                                         │
" │ 🔍 Similar Code (vector search):        │
" │  [x] 15 matches in codebase             │
" │  [x] 3 matches from git history         │
" │                                         │
" │ Total context: 247KB / 50KB max         │
" │ <Space> toggle | <Enter> preview        │
" └─────────────────────────────────────────┘
```

**Force regeneration with specific context:**

```vim
:MCPGenerate filesystem github
" Generates suggestion using only filesystem and github MCP servers
" Shows: "🔄 Generating with filesystem, github context..."
```

### 6. Troubleshooting & Transparency

**Connection status dashboard:**

```vim
:MCPStatus
" ┌─────────────────────────────────────────┐
" │ MCP Hub Status                          │
" ├─────────────────────────────────────────┤
" │ Rust Binary: ✓ Running (PID: 12345)     │
" │ Uptime: 2h 34m                          │
" │                                         │
" │ Connected Servers:                      │
" │  ✓ filesystem  [stdio]  45ms avg        │
" │  ✓ github      [http]   120ms avg       │
" │  ⚠ fetch       [stdio]  timeout         │
" │                                         │
" │ Performance:                            │
" │  Suggestions: 1,234 total               │
" │  Avg latency: 87ms                      │
" │  Cache hits: 67%                        │
" │                                         │
" │ Recent Errors:                          │
" │  [12:34] fetch timeout (3x)             │
" │  [11:20] github rate limit              │
" │                                         │
" │ [Reconnect] [View Logs] [Restart]       │
" └─────────────────────────────────────────┘
```

**Diagnostic logging:**

```vim
:MCPLogs
" Opens split with live log stream:
" [12:34:56] DEBUG Context gathered: filesystem=15KB, github=32KB
" [12:34:56] INFO  Generating completion (87% confidence)
" [12:34:56] DEBUG LLM API call: 247ms
" [12:34:57] INFO  Suggestion shown (total: 334ms)
```

### 7. Customization & Power User Features

**Configuration file example:**

```lua
-- ~/.config/nvim/lua/mcp_config.lua
require('mcp').setup({
  -- Behavior
  auto_trigger = true,
  trigger_delay = 200, -- ms
  
  -- Visual
  ghost_text = {
    enabled = true,
    hl_group = "Comment",
    fade_in = true,
  },
  
  -- Context
  max_context_size = 50 * 1024, -- 50KB
  context_sources = {
    current_buffer = true,
    open_buffers = true,
    filesystem = true,
    git_history = true,
    web_docs = false, -- disabled by default
  },
  
  -- Performance
  cache_enabled = true,
  cache_ttl = 300, -- seconds
  max_concurrent_requests = 3,
  
  -- MCP servers config
  config_file = "~/.config/mcp/servers.json",
  
  -- Keymaps
  keymaps = {
    accept = "<Tab>",
    accept_word = "<C-]>",
    accept_line = "<C-[>",
    next = "<C-n>",
    prev = "<C-p>",
    dismiss = "<Esc>",
    show_context = "<Leader>mc",
    generate_multi = "<Leader>mg",
  },
  
  -- Advanced: custom context filters
  context_filter = function(context)
    -- User can filter what context to include
    return context
  end,
})
```

## UX Principles Summary

1. **Non-intrusive**: Suggestions appear naturally, like thoughts completing themselves
2. **Transparent**: Always clear what context is being used and why
3. **Fast**: <100ms latency target for inline suggestions
4. **Configurable**: From simple defaults to power-user customization
5. **Debuggable**: Clear status, logs, and diagnostics when things go wrong
6. **Native feel**: Uses Neovim idioms (keybindings, visual language, commands)
7. **Context-aware**: Intelligently uses MCP servers for relevant suggestions
8. **Progressive disclosure**: Simple by default, powerful when needed

**Key differentiators from traditional autocomplete:**
- Uses MCP ecosystem for rich, dynamic context
- RAG-powered suggestions from entire codebase history
- Multi-server orchestration (filesystem + git + docs)
- Transparent about AI confidence and context sources
- Rust-powered performance for real-time suggestions

