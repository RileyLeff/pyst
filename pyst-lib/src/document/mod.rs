use anyhow::Result;

pub struct Documenter {}

impl Documenter {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn document(&self, _script: &str, _write: bool, _check: bool) -> Result<()> {
        Ok(())
    }
}