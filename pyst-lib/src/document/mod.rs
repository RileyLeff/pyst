use crate::config::Config;
use crate::introspection::runner::IntrospectionRunner;
use crate::introspection::schema::IntrospectionResult;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{self, Write};

#[derive(Debug, Serialize)]
struct DocumentationRequest {
    config: DocumentConfig,
    script_data: ScriptData,
}

#[derive(Debug, Serialize)]
struct DocumentConfig {
    model: String,
    api_key_env: String,
    api_base: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct ScriptData {
    script_content: String,
    entry_point: String,
    functions: Vec<FunctionInfo>,
    dependencies: Vec<String>,
    current_description: String,
    max_length: u32,
}

#[derive(Debug, Serialize)]
struct FunctionInfo {
    name: String,
    docstring: Option<String>,
    line_number: u32,
}

#[derive(Debug, Deserialize)]
struct DocumentationResponse {
    success: bool,
    description: Option<String>,
    error: Option<String>,
}

pub struct Documenter {
    config: Config,
}

impl Documenter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn document(&self, script_path: &str, write: bool, check: bool) -> Result<String> {
        let script_path = PathBuf::from(script_path);
        
        if !script_path.exists() {
            return Err(anyhow!("Script file not found: {}", script_path.display()));
        }
        
        // Check if documentation should be ignored
        if self.should_ignore_file(&script_path)? {
            return Err(anyhow!("Documentation is disabled for this file"));
        }
        
        // Get script introspection data
        let mut introspection_runner = IntrospectionRunner::new(self.config.clone())?;
        let introspection = introspection_runner.introspect(&script_path)?;
        
        // Read script content
        let script_content = std::fs::read_to_string(&script_path)?;
        
        // If --check flag is used, verify existing documentation
        if check {
            return self.check_documentation(&introspection).await;
        }
        
        // Generate new documentation
        let new_description = self.generate_description(&script_content, &introspection).await?;
        
        // If --write flag is used, write directly
        if write {
            self.write_description_to_file(&script_path, &new_description)?;
            return Ok(format!("Documentation written to {}", script_path.display()));
        }
        
        // Otherwise, show interactive diff
        self.interactive_documentation(&script_path, &introspection, &new_description).await
    }
    
    async fn generate_description(&self, script_content: &str, introspection: &IntrospectionResult) -> Result<String> {
        // Prepare script data
        let functions: Vec<FunctionInfo> = introspection.metadata.functions.iter().map(|f| {
            FunctionInfo {
                name: f.name.clone(),
                docstring: f.docstring.clone(),
                line_number: f.line_number,
            }
        }).collect();
        
        let dependencies: Vec<String> = introspection.metadata.dependencies.iter()
            .map(|d| d.name.clone()).collect();
        
        let entry_point = format!("{:?}", introspection.metadata.entry_points);
        let current_description = introspection.metadata.description.clone().unwrap_or_default();
        
        let script_data = ScriptData {
            script_content: script_content.to_string(),
            entry_point,
            functions,
            dependencies,
            current_description,
            max_length: 80, // Terminal-friendly length
        };
        
        // Prepare config
        let doc_config = DocumentConfig {
            model: self.config.document.model.clone(),
            api_key_env: self.config.document.api_key_env.clone(),
            api_base: self.config.document.api_base.clone(),
            max_tokens: self.config.document.max_tokens,
            temperature: self.config.document.temperature,
        };
        
        let request = DocumentationRequest {
            config: doc_config,
            script_data,
        };
        
        // Call the documenter.py helper
        self.call_documenter_helper(&request).await
    }
    
    async fn call_documenter_helper(&self, request: &DocumentationRequest) -> Result<String> {
        let request_json = serde_json::to_string(request)?;
        
        // Find the documenter.py helper
        let helper_path = self.find_documenter_helper()?;
        
        // Execute the helper with uv run
        let output = Command::new("uv")
            .args(&["run", &helper_path.to_string_lossy(), &request_json])
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Documenter helper failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: DocumentationResponse = serde_json::from_str(&stdout)?;
        
        if !response.success {
            return Err(anyhow!("Documentation generation failed: {}", 
                response.error.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        response.description.ok_or_else(|| anyhow!("No description generated"))
    }
    
    fn find_documenter_helper(&self) -> Result<PathBuf> {
        // Look for documenter.py in the helpers directory relative to the library
        let current_exe = std::env::current_exe()?;
        let exe_dir = current_exe.parent().ok_or_else(|| anyhow!("Cannot find executable directory"))?;
        
        // Try different possible locations
        let possible_paths = vec![
            exe_dir.join("../pyst-lib/helpers/documenter.py"),
            exe_dir.join("../../pyst-lib/helpers/documenter.py"),
            exe_dir.join("../../../pyst-lib/helpers/documenter.py"),
            PathBuf::from("pyst-lib/helpers/documenter.py"),
            PathBuf::from("helpers/documenter.py"),
        ];
        
        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(anyhow!("Cannot find documenter.py helper script"))
    }
    
    async fn check_documentation(&self, introspection: &IntrospectionResult) -> Result<String> {
        match &introspection.metadata.description {
            Some(desc) if !desc.trim().is_empty() => {
                Ok(format!("✅ Documentation exists: {}", desc))
            }
            _ => {
                Err(anyhow!("❌ No documentation found"))
            }
        }
    }
    
    async fn interactive_documentation(&self, script_path: &Path, introspection: &IntrospectionResult, new_description: &str) -> Result<String> {
        let current_description = introspection.metadata.description.as_deref().unwrap_or("(none)");
        
        println!("📝 Documentation Proposal for {}", script_path.display());
        println!();
        println!("Current: {}", current_description);
        println!("Proposed: {}", new_description);
        println!();
        
        print!("Accept this description? [y/N]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                self.write_description_to_file(script_path, new_description)?;
                Ok(format!("✅ Documentation updated in {}", script_path.display()))
            }
            _ => {
                Ok("❌ Documentation update cancelled".to_string())
            }
        }
    }
    
    fn write_description_to_file(&self, script_path: &Path, description: &str) -> Result<()> {
        let content = std::fs::read_to_string(script_path)?;
        
        // Simple implementation: add/update a docstring at the top of the file
        // This is a basic version - a more sophisticated implementation would
        // parse the AST and update the existing docstring
        
        let new_content = if content.starts_with("#!/") {
            // Handle shebang
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() > 1 && lines[1].starts_with("\"\"\"") {
                // Replace existing docstring
                let mut new_lines = vec![lines[0].to_string()]; // Keep shebang
                new_lines.push(format!("\"\"\"{}\"\"\"", description));
                
                // Skip old docstring
                let mut i = 1;
                if i < lines.len() && lines[i].starts_with("\"\"\"") {
                    i += 1;
                    while i < lines.len() && !lines[i].ends_with("\"\"\"") {
                        i += 1;
                    }
                    if i < lines.len() {
                        i += 1; // Skip closing """
                    }
                }
                
                new_lines.extend(lines[i..].iter().map(|s| s.to_string()));
                new_lines.join("\n")
            } else {
                // Add new docstring after shebang
                let mut new_lines = vec![lines[0].to_string()]; // Keep shebang
                new_lines.push(format!("\"\"\"{}\"\"\"", description));
                new_lines.extend(lines[1..].iter().map(|s| s.to_string()));
                new_lines.join("\n")
            }
        } else {
            // Add docstring at the beginning
            format!("\"\"\"{}\"\"\"\\n{}", description, content)
        };
        
        std::fs::write(script_path, new_content)?;
        Ok(())
    }
    
    fn should_ignore_file(&self, script_path: &Path) -> Result<bool> {
        // Check for .pystdocignore file
        let mut current_dir = script_path.parent();
        
        while let Some(dir) = current_dir {
            let ignore_file = dir.join(".pystdocignore");
            if ignore_file.exists() {
                let ignore_content = std::fs::read_to_string(&ignore_file)?;
                let script_name = script_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                for line in ignore_content.lines() {
                    let pattern = line.trim();
                    if !pattern.is_empty() && !pattern.starts_with('#') {
                        if glob::Pattern::new(pattern)
                            .map_err(|e| anyhow!("Invalid pattern in .pystdocignore: {}", e))?
                            .matches(script_name) {
                            return Ok(true);
                        }
                    }
                }
            }
            
            current_dir = dir.parent();
        }
        
        // Check for inline markers in the file
        let content = std::fs::read_to_string(script_path)?;
        if content.contains("# pyst:doc:ignore") || content.contains("# pyst:nodoc") {
            return Ok(true);
        }
        
        Ok(false)
    }
}