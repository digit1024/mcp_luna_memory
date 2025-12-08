mod service;

use anyhow::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use service::ConversationService;

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = "/home/digit1024/.local/share/cosmic_llm/conversations.db";
    let service = ConversationService::new(db_path)?;

    let server = service.serve(stdio()).await
        .map_err(|e| {
            eprintln!("Error starting server: {:?}", e);
            e
        })?;
    
    server.waiting().await
        .map_err(|e| {
            eprintln!("Error waiting for server: {:?}", e);
            e
        })?;

    Ok(())
}
