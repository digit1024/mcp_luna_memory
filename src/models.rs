use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

// Conversation-related request types
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchConversationsRequest {
    #[schemars(description = "Keywords to search in conversation messages (OR semantics)")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetConversationRequest {
    #[schemars(description = "The unique identifier of the conversation to retrieve")]
    pub conversation_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchTitlesRequest {
    #[schemars(description = "Search query to find in conversation titles")]
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Request parameters for listing conversations")]
pub struct ListConversationsRequest {
    #[schemars(description = "Maximum number of conversations to return (default: 50, max: 200)")]
    pub limit: Option<u32>,
    #[schemars(description = "Number of conversations to skip (default: 0)")]
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetMessageRequest {
    #[schemars(description = "The unique identifier of the message to retrieve")]
    pub message_id: i64,
}

// Conversation-related response types
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Search result from conversation messages")]
pub struct SearchResult {
    pub conversation_id: String,
    pub message_id: i64,
    pub role: String,
    pub content_preview: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Wrapper for search results array")]
pub struct SearchResultsResponse {
    pub items: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub title_generated: i32,
    pub profile_name: Option<String>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub title_generated: i32,
    pub profile_name: Option<String>,
    pub message_count: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Wrapper for conversation summaries array")]
pub struct ConversationSummariesResponse {
    pub items: Vec<ConversationSummary>,
}

// Memory Module Types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StoreMemoryRequest {
    #[schemars(description = "The fact or information to remember")]
    pub content: String,
    #[schemars(description = "A tag for grouping (e.g., 'workflow', 'crate-info')")]
    pub category: Option<String>,
    #[schemars(description = "Priority score 1-10 (default: 5)")]
    pub importance: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchMemoryRequest {
    #[schemars(description = "Keywords to search in memory (OR semantics)")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchMemoryByCategoryRequest {
    #[schemars(description = "Category to filter memory entries (e.g. 'moltbook', 'work', 'personal')")]
    pub category: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteMemoryRequest {
    #[schemars(description = "The ID of the memory entry to remove")]
    pub memory_id: i64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryEntry {
    pub id: i64,
    pub content: String,
    pub category: Option<String>,
    pub importance: i32,
    pub created_at: i64,
}

#[derive(Debug, Serialize, JsonSchema)]
#[schemars(description = "Wrapper for memory search results array")]
pub struct MemorySearchResponse {
    pub items: Vec<MemoryEntry>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DeleteMemoryResponse {
    pub success: bool,
    #[schemars(description = "Error message if deletion failed")]
    pub error: Option<String>,
}

