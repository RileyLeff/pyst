use anyhow::Result;
use axum::{routing::get, Router};

pub struct McpServer {}

impl McpServer {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn start(&self, _port: u16) -> Result<()> {
        let _app: Router = Router::new().route("/", get(|| async { "MCP Server" }));
        
        println!("MCP server would start here");
        Ok(())
    }
}