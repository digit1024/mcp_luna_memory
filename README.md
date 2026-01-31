<div align="center">
  <br>
  <h1>🌙 MCP Luna History</h1>

  <p><strong>MCP server for Luna AI – conversation history &amp; memory</strong></p>
  
  <p>Built specifically for <a href="https://github.com/digit1024/LunaAI">Luna AI</a>. Provides access to past conversations stored in Cosmic LLM's SQLite database, plus long-term memory persistence for user preferences, technical setups, and important facts.</p>
</div>

## About

This MCP (Model Context Protocol) server is designed for **[Luna AI](https://github.com/digit1024/LunaAI)** – your brilliant AI companion for desktop and mobile. It enables Luna to search and retrieve conversation history using full-text search, and to store and recall knowledge across sessions.

## Features

- **Full-text search** across all conversation messages
- **Retrieve complete conversation threads** with all messages in chronological order
- **Search conversation titles** to quickly find topics
- **List conversations** with pagination support
- **Get individual messages** by message ID
- **Memory persistence** – store, search, and delete long-term knowledge
- **Search memory by category** – filter by tags like `work`, `personal`, `moltbook`, `security`

## Tools

### `search_conversations`
Search across all past conversations with the user using full-text search. Keywords are combined with OR semantics.

**Parameters:**
- `keywords` (array of strings): Keywords to search in conversation messages

### `get_conversation`
Retrieve a complete conversation thread from past conversations with the user. Returns the full conversation including all messages, tool calls, and responses in chronological order.

**Parameters:**
- `conversation_id` (string): The unique identifier of the conversation to retrieve

### `search_conversation_titles`
Search conversation titles from past conversations with the user. This tool helps you quickly find conversations by their titles when you remember the topic but not the exact conversation ID.

**Parameters:**
- `query` (string): Search query to find in conversation titles

### `list_conversations`
List past conversations with the user, ordered by most recent. Useful for browsing conversation history and finding conversations by recency.

**Parameters:**
- `limit` (integer, optional): Maximum number of conversations to return (default: 50, max: 200)
- `offset` (integer, optional): Number of conversations to skip (default: 0)

### `get_message`
Retrieve a specific message from past conversations with the user by its message ID. Returns the complete message including content, role, tool calls, and any associated metadata.

**Parameters:**
- `message_id` (integer): The unique identifier of the message to retrieve

### `store_memory`
Store important facts, preferences, or relevant information in long-term memory.

**Parameters:**
- `content` (string): The fact or information to remember
- `category` (string, optional): A tag for grouping (e.g. `workflow`, `moltbook`, `personal`)
- `importance` (integer, optional): Priority score 1–10 (default: 5)

### `search_memory`
Search long-term memory using full-text search. Keywords are combined with OR semantics. Results are ranked by relevance (BM25).

**Parameters:**
- `keywords` (array of strings): Keywords to search in memory

### `search_memory_by_category`
Search memory entries by category. Returns all entries in the given category, ordered by importance and recency.

**Parameters:**
- `category` (string): Category to filter (e.g. `moltbook`, `work`, `personal`, `security`)

### `delete_memory`
Delete a memory entry by its ID. Use to remove outdated or incorrect information.

**Parameters:**
- `memory_id` (integer): The ID of the memory entry to remove

## Building

```bash
cargo build --release
```

## Running

The server communicates via stdio (standard input/output) and requires the `COSMIC_LLM_DB_PATH` environment variable to be set:

```bash
export COSMIC_LLM_DB_PATH="/path/to/conversations.db"
./target/release/mcp_luna_history
```

Or using cargo:

```bash
export COSMIC_LLM_DB_PATH="/path/to/conversations.db"
cargo run --release
```

## Database

The server connects to a SQLite database specified by the `COSMIC_LLM_DB_PATH` environment variable.

**Required Environment Variable:**
- `COSMIC_LLM_DB_PATH`: Path to the SQLite database file containing conversation history

**Example:**
```bash
export COSMIC_LLM_DB_PATH="/home/digit1024/.local/share/cosmic_llm/conversations.db"
./target/release/mcp_luna_history
```

The database must contain:
- `conversations` table with conversation metadata
- `messages` table with message content
- `messages_fts` FTS5 virtual table for conversation full-text search
- `memory` table (created on first use) for long-term storage
- `memory_fts` FTS5 virtual table for memory full-text search

## MCP Client Configuration

### Luna AI

Add this server to Luna AI's MCP configuration. Luna stores its conversation database at `~/.local/share/cosmic_llm/conversations.db` (Linux).

```json
{
  "mcpServers": {
    "luna-history": {
      "command": "/path/to/mcp_luna_history/target/release/mcp_luna_history",
      "args": [],
      "env": {
        "COSMIC_LLM_DB_PATH": "/home/USER/.local/share/cosmic_llm/conversations.db"
      }
    }
  }
}
```

### Other MCP Clients

Works with any MCP client that supports stdio transport (e.g. Claude Desktop):

```json
{
  "mcpServers": {
    "cosmic-llm-history": {
      "command": "/path/to/mcp_luna_history/target/release/mcp_luna_history",
      "args": [],
      "env": {
        "COSMIC_LLM_DB_PATH": "/path/to/conversations.db"
      }
    }
  }
}
```

## License

MIT

---

<div align="center">
  <p><strong>Part of the Luna AI ecosystem</strong></p>
  <p><a href="https://github.com/digit1024/LunaAI">🌙 Luna AI</a> – Your brilliant AI companion for desktop and mobile</p>
</div>

