use anyhow::{Context, Result};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::{Json, Parameters}},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub struct ConversationService {
    db: Arc<Mutex<Connection>>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchConversationsRequest {
    #[schemars(description = "Search query to find in conversation messages")]
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetConversationRequest {
    #[schemars(description = "The unique identifier of the conversation to retrieve")]
    pub conversation_id: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchTitlesRequest {
    #[schemars(description = "Search query to find in conversation titles")]
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListConversationsRequest {
    #[schemars(description = "Maximum number of conversations to return (default: 50, max: 200)")]
    pub limit: Option<u32>,
    #[schemars(description = "Number of conversations to skip (default: 0)")]
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetMessageRequest {
    #[schemars(description = "The unique identifier of the message to retrieve")]
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(schemars::JsonSchema)]
#[schemars(description = "Search result from conversation messages")]
pub struct SearchResult {
    pub conversation_id: String,
    pub message_id: i64,
    pub role: String,
    pub content_preview: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub title_generated: i32,
    pub profile_name: Option<String>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Message {
    pub id: i64,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
    pub tool_calls: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_status: Option<String>,
    pub tool_params_json: Option<String>,
    pub tool_result_json: Option<String>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub title_generated: i32,
    pub profile_name: Option<String>,
    pub message_count: i64,
}

#[tool_router]
impl ConversationService {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "Search across all past conversations with the user using full-text search. This tool searches through message content in all conversation history, allowing you to find relevant past discussions based on keywords or phrases.")]
    pub fn search_conversations(
        &self,
        Parameters(SearchConversationsRequest { query }): Parameters<SearchConversationsRequest>,
    ) -> Json<Vec<SearchResult>> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to lock database: {}", e);
                return Json(Vec::new());
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
            Err(e) => {
                eprintln!("Database error preparing statement: {}", e);
                return Json(Vec::new());
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
                    Err(e) => {
                        eprintln!("Database error collecting results: {}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Database error executing query: {}", e);
                Vec::new()
            }
        };

        Json(results)
    }

    #[tool(description = "Retrieve a complete conversation thread from past conversations with the user. Returns the full conversation including all messages, tool calls, and responses in chronological order. Returns empty object if not found.")]
    pub fn get_conversation(
        &self,
        Parameters(GetConversationRequest { conversation_id }): Parameters<GetConversationRequest>,
    ) -> Json<Conversation> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to lock database: {}", e);
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
            Err(e) => {
                eprintln!("Database error preparing statement: {}", e);
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
            Err(e) => {
                eprintln!("Database error querying conversation: {}", e);
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
            Err(e) => {
                eprintln!("Database error preparing statement: {}", e);
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
                    Err(e) => {
                        eprintln!("Database error collecting messages: {}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Database error executing query: {}", e);
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
    ) -> Json<Vec<ConversationSummary>> {
        let search_pattern = format!("%{}%", query);
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to lock database: {}", e);
                return Json(Vec::new());
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
            Err(e) => {
                eprintln!("Database error preparing statement: {}", e);
                return Json(Vec::new());
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
                    Err(e) => {
                        eprintln!("Database error collecting results: {}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Database error executing query: {}", e);
                Vec::new()
            }
        };

        Json(results)
    }

    #[tool(description = "List past conversations with the user, ordered by most recent. Useful for browsing conversation history and finding conversations by recency.")]
    pub fn list_conversations(
        &self,
        Parameters(ListConversationsRequest { limit, offset }): Parameters<ListConversationsRequest>,
    ) -> Json<Vec<ConversationSummary>> {
        let limit = limit.unwrap_or(50).min(200) as i64;
        let offset = offset.unwrap_or(0) as i64;

        let db = match self.db.lock() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to lock database: {}", e);
                return Json(Vec::new());
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
            Err(e) => {
                eprintln!("Database error preparing statement: {}", e);
                return Json(Vec::new());
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
                    Err(e) => {
                        eprintln!("Database error collecting results: {}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Database error executing query: {}", e);
                Vec::new()
            }
        };

        Json(results)
    }

    #[tool(description = "Retrieve a specific message from past conversations with the user by its message ID. Returns the complete message including content, role, tool calls, and any associated metadata. Returns empty message if not found.")]
    pub fn get_message(
        &self,
        Parameters(GetMessageRequest { message_id }): Parameters<GetMessageRequest>,
    ) -> Json<Message> {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to lock database: {}", e);
                return Json(Message {
                    id: message_id,
                    conversation_id: "ERROR".to_string(),
                    role: "error".to_string(),
                    content: format!("Database lock error: {}", e),
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
            Err(e) => {
                eprintln!("Database error preparing statement: {}", e);
                return Json(Message {
                    id: message_id,
                    conversation_id: "ERROR".to_string(),
                    role: "error".to_string(),
                    content: format!("Database error: {}", e),
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
            Err(e) => {
                eprintln!("Database error querying message: {}", e);
                Json(Message {
                    id: message_id,
                    conversation_id: "ERROR".to_string(),
                    role: "error".to_string(),
                    content: format!("Database error: {}", e),
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
}

#[tool_handler]
impl ServerHandler for ConversationService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("MCP server for searching and retrieving past conversations with the user from Cosmic LLM history.".to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
