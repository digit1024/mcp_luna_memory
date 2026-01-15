mod db;
mod models;
mod service;

use anyhow::{Context, Result};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use service::ConversationService;

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = std::env::var("COSMIC_LLM_DB_PATH")
        .context("COSMIC_LLM_DB_PATH environment variable must be set")?;
    let service = ConversationService::new(&db_path)?;

    let server = service.serve(stdio()).await?;
    
    server.waiting().await?;

    Ok(())
}
