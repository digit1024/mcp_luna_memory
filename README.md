# MCP Server for Cosmic LLM Conversation History

An MCP (Model Context Protocol) server that provides access to past conversations stored in Cosmic LLM's SQLite database. This server allows you to search and retrieve conversation history using full-text search capabilities.

## Features

- **Full-text search** across all conversation messages
- **Retrieve complete conversation threads** with all messages in chronological order
- **Search conversation titles** to quickly find topics
- **List conversations** with pagination support
- **Get individual messages** by message ID

## Tools

### `search_conversations`
Search across all past conversations with the user using full-text search. This tool searches through message content in all conversation history, allowing you to find relevant past discussions based on keywords or phrases.

**Parameters:**
- `query` (string): Search query to find in conversation messages

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

## Building

```bash
cargo build --release
```

## Running

The server communicates via stdio (standard input/output):

```bash
./target/release/mcp_luna_history
```

Or using cargo:

```bash
cargo run --release
```

## Database

The server connects to the SQLite database at:
```
/home/digit1024/.local/share/cosmic_llm/conversations.db
```

The database must contain:
- `conversations` table with conversation metadata
- `messages` table with message content
- `messages_fts` FTS5 virtual table for full-text search

## MCP Client Configuration

To use this server with an MCP client, configure it to run this binary with stdio transport.

Example configuration (for Claude Desktop or similar):

```json
{
  "mcpServers": {
    "cosmic-llm-history": {
      "command": "/path/to/mcp_luna_history/target/release/mcp_luna_history",
      "args": []
    }
  }
}
```

## License

MIT

