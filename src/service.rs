use anyhow::{Context, Result};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::{Json, Parameters}},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::db;
use crate::models::*;

pub struct ConversationService {
    db: Arc<Mutex<Connection>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router(router = tool_router)]
impl ConversationService {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;

        // Initialize memory module schema
        db::init_memory_schema(&conn)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "Search across all past conversations with the user using full-text search. This tool searches through message content in all conversation history, allowing you to find relevant past discussions based on keywords or phrases.")]
    pub fn search_conversations(
        &self,
        Parameters(SearchConversationsRequest { query }): Parameters<SearchConversationsRequest>,
    ) -> Json<SearchResultsResponse> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(SearchResultsResponse { items: Vec::new() });
            }
        };
        
        let mut stmt = match db.prepare(
            r#"
            SELECT DISTINCT
                m.id,
                m.conversation_id,
                m.role,
                substr(m.content, 1, 200) as content_preview,
                m.created_at
            FROM messages m
            JOIN messages_fts ON m.id = messages_fts.rowid
            WHERE messages_fts MATCH ?
            ORDER BY m.created_at DESC
            LIMIT 50
            "#
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(SearchResultsResponse { items: Vec::new() });
            }
        };

        let results: Vec<SearchResult> = match stmt.query_map([query.as_str()], |row| {
            Ok(SearchResult {
                message_id: row.get(0).unwrap_or(0),
                conversation_id: row.get(1).unwrap_or_default(),
                role: row.get(2).unwrap_or_default(),
                content_preview: row.get(3).unwrap_or_default(),
                created_at: row.get(4).unwrap_or(0),
            })
        }) {
            Ok(iter) => {
                match iter.collect::<Result<Vec<_>, _>>() {
                    Ok(results) => results,
                    Err(_) => {
                        Vec::new()
                    }
                }
            }
            Err(_) => {
                Vec::new()
            }
        };

        Json(SearchResultsResponse { items: results })
    }

    #[tool(description = "Retrieve a complete conversation thread from past conversations with the user. Returns the full conversation including all messages, tool calls, and responses in chronological order. Returns empty object if not found.")]
    pub fn get_conversation(
        &self,
        Parameters(GetConversationRequest { conversation_id }): Parameters<GetConversationRequest>,
    ) -> Json<Conversation> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(Conversation {
                    id: conversation_id.clone(),
                    title: "ERROR".to_string(),
                    created_at: 0,
                    title_generated: 0,
                    profile_name: None,
                    messages: Vec::new(),
                });
            }
        };
        
        // Get conversation metadata
        let mut conv_stmt = match db.prepare(
            "SELECT id, title, created_at, title_generated, profile_name FROM conversations WHERE id = ?"
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(Conversation {
                    id: conversation_id.clone(),
                    title: "ERROR".to_string(),
                    created_at: 0,
                    title_generated: 0,
                    profile_name: None,
                    messages: Vec::new(),
                });
            }
        };

        let mut conversation: Conversation = match conv_stmt
            .query_row([conversation_id.as_str()], |row| {
                Ok(Conversation {
                    id: row.get(0).unwrap_or_default(),
                    title: row.get(1).unwrap_or_default(),
                    created_at: row.get(2).unwrap_or(0),
                    title_generated: row.get(3).unwrap_or(0),
                    profile_name: row.get(4).ok(),
                    messages: Vec::new(),
                })
            }) {
            Ok(conv) => conv,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Return empty conversation with error indicator
                return Json(Conversation {
                    id: conversation_id.clone(),
                    title: "NOT_FOUND".to_string(),
                    created_at: 0,
                    title_generated: 0,
                    profile_name: None,
                    messages: Vec::new(),
                });
            }
            Err(_) => {
                return Json(Conversation {
                    id: conversation_id.clone(),
                    title: "ERROR".to_string(),
                    created_at: 0,
                    title_generated: 0,
                    profile_name: None,
                    messages: Vec::new(),
                });
            }
        };

        // Get all messages for this conversation
        let mut msg_stmt = match db.prepare(
            r#"
            SELECT 
                id, conversation_id, role, content, created_at,
                tool_calls, tool_call_id, tool_name, tool_status,
                tool_params_json, tool_result_json, reasoning_content
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at ASC
            "#
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(conversation);
            }
        };

        let messages: Vec<Message> = match msg_stmt.query_map([conversation_id.as_str()], |row| {
            Ok(Message {
                id: row.get(0).unwrap_or(0),
                conversation_id: row.get(1).unwrap_or_default(),
                role: row.get(2).unwrap_or_default(),
                content: row.get(3).unwrap_or_default(),
                created_at: row.get(4).unwrap_or(0),
                tool_calls: row.get(5).ok(),
                tool_call_id: row.get(6).ok(),
                tool_name: row.get(7).ok(),
                tool_status: row.get(8).ok(),
                tool_params_json: row.get(9).ok(),
                tool_result_json: row.get(10).ok(),
                reasoning_content: row.get(11).ok(),
            })
        }) {
            Ok(iter) => {
                match iter.collect::<Result<Vec<_>, _>>() {
                    Ok(messages) => messages,
                    Err(_) => {
                        Vec::new()
                    }
                }
            }
            Err(_) => {
                Vec::new()
            }
        };

        conversation.messages = messages;
        Json(conversation)
    }

    #[tool(description = "Search conversation titles from past conversations with the user. This tool helps you quickly find conversations by their titles when you remember the topic but not the exact conversation ID.")]
    pub fn search_conversation_titles(
        &self,
        Parameters(SearchTitlesRequest { query }): Parameters<SearchTitlesRequest>,
    ) -> Json<ConversationSummariesResponse> {
        let search_pattern = format!("%{}%", query);
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(ConversationSummariesResponse { items: Vec::new() });
            }
        };
        
        let mut stmt = match db.prepare(
            r#"
            SELECT 
                c.id,
                c.title,
                c.created_at,
                c.title_generated,
                c.profile_name,
                COUNT(m.id) as message_count
            FROM conversations c
            LEFT JOIN messages m ON c.id = m.conversation_id
            WHERE c.title LIKE ?
            GROUP BY c.id, c.title, c.created_at, c.title_generated, c.profile_name
            ORDER BY c.created_at DESC
            LIMIT 100
            "#
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(ConversationSummariesResponse { items: Vec::new() });
            }
        };

        let results: Vec<ConversationSummary> = match stmt.query_map([&search_pattern], |row| {
            Ok(ConversationSummary {
                id: row.get(0).unwrap_or_default(),
                title: row.get(1).unwrap_or_default(),
                created_at: row.get(2).unwrap_or(0),
                title_generated: row.get(3).unwrap_or(0),
                profile_name: row.get(4).ok(),
                message_count: row.get(5).unwrap_or(0),
            })
        }) {
            Ok(iter) => {
                match iter.collect::<Result<Vec<_>, _>>() {
                    Ok(results) => results,
                    Err(_) => {
                        Vec::new()
                    }
                }
            }
            Err(_) => {
                Vec::new()
            }
        };

        Json(ConversationSummariesResponse { items: results })
    }

    #[tool(description = "List past conversations with the user, ordered by most recent. Useful for browsing conversation history and finding conversations by recency.")]
    pub fn list_conversations(
        &self,
        Parameters(ListConversationsRequest { limit, offset }): Parameters<ListConversationsRequest>,
    ) -> Json<ConversationSummariesResponse> {
        let limit = limit.unwrap_or(50).min(200) as i64;
        let offset = offset.unwrap_or(0) as i64;

        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(ConversationSummariesResponse { items: Vec::new() });
            }
        };
        
        let mut stmt = match db.prepare(
            r#"
            SELECT 
                c.id,
                c.title,
                c.created_at,
                c.title_generated,
                c.profile_name,
                COUNT(m.id) as message_count
            FROM conversations c
            LEFT JOIN messages m ON c.id = m.conversation_id
            GROUP BY c.id, c.title, c.created_at, c.title_generated, c.profile_name
            ORDER BY c.created_at DESC
            LIMIT ? OFFSET ?
            "#
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(ConversationSummariesResponse { items: Vec::new() });
            }
        };

        let results: Vec<ConversationSummary> = match stmt.query_map([limit, offset], |row| {
            Ok(ConversationSummary {
                id: row.get(0).unwrap_or_default(),
                title: row.get(1).unwrap_or_default(),
                created_at: row.get(2).unwrap_or(0),
                title_generated: row.get(3).unwrap_or(0),
                profile_name: row.get(4).ok(),
                message_count: row.get(5).unwrap_or(0),
            })
        }) {
            Ok(iter) => {
                match iter.collect::<Result<Vec<_>, _>>() {
                    Ok(results) => results,
                    Err(_) => {
                        Vec::new()
                    }
                }
            }
            Err(_) => {
                Vec::new()
            }
        };

        Json(ConversationSummariesResponse { items: results })
    }

    #[tool(description = "Retrieve a specific message from past conversations with the user by its message ID. Returns the complete message including content, role, tool calls, and any associated metadata. Returns empty message if not found.")]
    pub fn get_message(
        &self,
        Parameters(GetMessageRequest { message_id }): Parameters<GetMessageRequest>,
    ) -> Json<Message> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(Message {
                    id: message_id,
                    conversation_id: "ERROR".to_string(),
                    role: "error".to_string(),
                    content: "Database lock error".to_string(),
                    created_at: 0,
                    tool_calls: None,
                    tool_call_id: None,
                    tool_name: None,
                    tool_status: None,
                    tool_params_json: None,
                    tool_result_json: None,
                    reasoning_content: None,
                });
            }
        };
        
        let mut stmt = match db.prepare(
            r#"
            SELECT 
                id, conversation_id, role, content, created_at,
                tool_calls, tool_call_id, tool_name, tool_status,
                tool_params_json, tool_result_json, reasoning_content
            FROM messages
            WHERE id = ?
            "#
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(Message {
                    id: message_id,
                    conversation_id: "ERROR".to_string(),
                    role: "error".to_string(),
                    content: "Database error".to_string(),
                    created_at: 0,
                    tool_calls: None,
                    tool_call_id: None,
                    tool_name: None,
                    tool_status: None,
                    tool_params_json: None,
                    tool_result_json: None,
                    reasoning_content: None,
                });
            }
        };

        match stmt.query_row([message_id], |row| {
            Ok(Message {
                id: row.get(0).unwrap_or(0),
                conversation_id: row.get(1).unwrap_or_default(),
                role: row.get(2).unwrap_or_default(),
                content: row.get(3).unwrap_or_default(),
                created_at: row.get(4).unwrap_or(0),
                tool_calls: row.get(5).ok(),
                tool_call_id: row.get(6).ok(),
                tool_name: row.get(7).ok(),
                tool_status: row.get(8).ok(),
                tool_params_json: row.get(9).ok(),
                tool_result_json: row.get(10).ok(),
                reasoning_content: row.get(11).ok(),
            })
        }) {
            Ok(msg) => Json(msg),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Return empty message with error indicator
                Json(Message {
                    id: message_id,
                    conversation_id: "NOT_FOUND".to_string(),
                    role: "error".to_string(),
                    content: "Message not found".to_string(),
                    created_at: 0,
                    tool_calls: None,
                    tool_call_id: None,
                    tool_name: None,
                    tool_status: None,
                    tool_params_json: None,
                    tool_result_json: None,
                    reasoning_content: None,
                })
            }
            Err(_) => {
                Json(Message {
                    id: message_id,
                    conversation_id: "ERROR".to_string(),
                    role: "error".to_string(),
                    content: "Database error".to_string(),
                    created_at: 0,
                    tool_calls: None,
                    tool_call_id: None,
                    tool_name: None,
                    tool_status: None,
                    tool_params_json: None,
                    tool_result_json: None,
                    reasoning_content: None,
                })
            }
        }
    }

    #[tool(description = "THIS IS A TOOL TO REMEMBER, OR TO UPDATE(Delete and then create) THE MEMORY.USE IT OFTEN TO REMEMBER IMPORTANT STUFF! Store important facts, preferences, or relevant information in long-term memory.")]
    pub fn store_memory(
        &self,
        Parameters(StoreMemoryRequest {
            content,
            category,
            importance,
        }): Parameters<StoreMemoryRequest>,
    ) -> Json<MemoryEntry> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(MemoryEntry {
                    id: 0,
                    content: "Database lock error".to_string(),
                    category: None,
                    importance: 0,
                    created_at: 0,
                });
            }
        };

        let importance_value = importance.unwrap_or(5);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        match db.execute(
            "INSERT INTO memory (content, category, importance, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![content, category, importance_value, created_at],
        ) {
            Ok(_) => {
                let id = db.last_insert_rowid();
                Json(MemoryEntry {
                    id,
                    content,
                    category,
                    importance: importance_value,
                    created_at,
                })
            }
            Err(e) => {
                Json(MemoryEntry {
                    id: 0,
                    content: format!("Failed to store memory: {}", e),
                    category: None,
                    importance: 0,
                    created_at: 0,
                })
            }
        }
    }

    #[tool(description = "THISI IS A TOOL TO REMIND/GET CONTEXT FROM MEMORY. USE IT OFTEN! USE IT TOGETHER WITH CHAT HISTORY IF NEEDED (for details) ! Search long-term memory using full-text search. This tool finds relevant stored knowledge based on keywords or phrases. Results are ranked by relevance.")]
    pub fn search_memory(
        &self,
        Parameters(SearchMemoryRequest { query }): Parameters<SearchMemoryRequest>,
    ) -> Json<MemorySearchResponse> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return Json(MemorySearchResponse { items: Vec::new() });
            }
        };

        let mut stmt = match db.prepare(
            r#"
            SELECT 
                m.id,
                m.content,
                m.category,
                m.importance,
                m.created_at
            FROM memory m
            JOIN memory_fts ON m.id = memory_fts.rowid
            WHERE memory_fts MATCH ?
            ORDER BY bm25(memory_fts) ASC
            LIMIT 10
            "#
        ) {
            Ok(stmt) => stmt,
            Err(_) => {
                return Json(MemorySearchResponse { items: Vec::new() });
            }
        };

        let results: Vec<MemoryEntry> = match stmt.query_map([query.as_str()], |row| {
            Ok(MemoryEntry {
                id: row.get(0).unwrap_or(0),
                content: row.get(1).unwrap_or_default(),
                category: row.get(2).ok(),
                importance: row.get(3).unwrap_or(5),
                created_at: row.get(4).unwrap_or(0),
            })
        }) {
            Ok(iter) => {
                match iter.collect::<Result<Vec<_>, _>>() {
                    Ok(results) => results,
                    Err(_) => {
                        Vec::new()
                    }
                }
            }
            Err(_) => {
                Vec::new()
            }
        };

        Json(MemorySearchResponse { items: results })
    }

    #[tool(description = "THIS IS A TOOL TO FORGET, OR TO UPDATE(Delete and then create) THE MEMORY USE IT TO CORRECT YOUR MEMORIES. Delete a memory entry by its ID. Use this to remove outdated or incorrect information from long-term memory.")]
    pub fn delete_memory(
        &self,
        Parameters(DeleteMemoryRequest { memory_id }): Parameters<DeleteMemoryRequest>,
    ) -> Json<DeleteMemoryResponse> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(e) => {
                return Json(DeleteMemoryResponse {
                    success: false,
                    error: Some(format!("Database lock error: {}", e)),
                });
            }
        };

        match db.execute("DELETE FROM memory WHERE id = ?", [memory_id]) {
            Ok(rows_affected) => {
                if rows_affected > 0 {
                    Json(DeleteMemoryResponse {
                        success: true,
                        error: None,
                    })
                } else {
                    Json(DeleteMemoryResponse {
                        success: false,
                        error: Some("Memory entry not found".to_string()),
                    })
                }
            }
            Err(e) => {
                Json(DeleteMemoryResponse {
                    success: false,
                    error: Some(format!("Failed to delete memory: {}", e)),
                })
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for ConversationService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("MCP server for searching and retrieving past conversations with the user from Cosmic LLM history. Also provides memory persistence capabilities - use search_memory to check for user preferences, technical setups, or important facts stored in previous conversations before answering questions.".to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
