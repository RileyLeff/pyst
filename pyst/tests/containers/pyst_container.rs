use testcontainers::{
    GenericImage, ImageExt, ContainerRequest, core::ExecCommand,
};

/// Helper functions for creating standardized Pyst test containers
pub struct PystContainer;

impl PystContainer {
    /// Create a new Pyst test container with Python 3.11
    pub fn python_base() -> ContainerRequest<GenericImage> {
        GenericImage::new("python", "3.11-slim")
            .with_cmd(vec!["sleep", "infinity"])
    }
    
    /// Create a Python container with basic environment setup for pyst testing
    pub fn python_with_env() -> ContainerRequest<GenericImage> {
        GenericImage::new("python", "3.11-slim")
            .with_cmd(vec!["sleep", "infinity"])
            .with_env_var("PYTHONUNBUFFERED", "1")
            .with_env_var("DEBIAN_FRONTEND", "noninteractive")
            .with_env_var("UV_NO_NETWORK", "0")  // Allow network for uv package installs
    }
}

/// Helper for executing commands and capturing output
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i64,
}

impl CommandResult {
    pub async fn exec(
        container: &testcontainers::ContainerAsync<GenericImage>, 
        cmd: Vec<&str>
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    {
        let exec_command = ExecCommand::new(cmd);
        let mut exec_result = container.exec(exec_command).await?;
        
        let stdout_bytes = exec_result.stdout_to_vec().await?;
        let stderr_bytes = exec_result.stderr_to_vec().await?;
        let exit_code = exec_result.exit_code().await?.unwrap_or(-1);
        
        Ok(CommandResult {
            stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
            stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
            exit_code,
        })
    }
    
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::runners::AsyncRunner;
    
    #[tokio::test]
    async fn test_container_creation() {
        let image = PystContainer::python_base();
        let container = image.start().await.expect("Failed to start container");
        
        // Test that we can execute commands
        let result = CommandResult::exec(&container, vec!["python", "--version"]).await
            .expect("Failed to execute command");
            
        assert!(result.success(), "Python version command should succeed");
        assert!(result.stdout.contains("Python 3.11"), "Should return Python 3.11 version");
        
        println!("✅ Basic Python container: {}", result.stdout.trim());
    }
    
    #[tokio::test]
    async fn test_container_with_env() {
        let image = PystContainer::python_with_env();
        let container = image.start().await.expect("Failed to start container");
        
        // Test that environment variables are set
        let result = CommandResult::exec(&container, vec!["env"]).await
            .expect("Failed to execute env command");
            
        assert!(result.success(), "Env command should succeed");
        assert!(result.stdout.contains("PYTHONUNBUFFERED=1"), "PYTHONUNBUFFERED should be set");
        assert!(result.stdout.contains("DEBIAN_FRONTEND=noninteractive"), "DEBIAN_FRONTEND should be set");
        
        println!("✅ Environment variables properly configured");
    }
    
    #[tokio::test]
    async fn test_basic_commands() {
        let image = PystContainer::python_base();
        let container = image.start().await.expect("Failed to start container");
        
        // Test python --version
        let py_result = CommandResult::exec(&container, vec!["python", "--version"]).await
            .expect("Failed to execute python --version");
        assert!(py_result.success());
        assert!(py_result.stdout.contains("Python"));
        
        // Test ls
        let ls_result = CommandResult::exec(&container, vec!["ls", "-la", "/"]).await
            .expect("Failed to execute ls");
        assert!(ls_result.success());
        assert!(ls_result.stdout.contains("bin"));
        
        // Test pwd
        let pwd_result = CommandResult::exec(&container, vec!["pwd"]).await
            .expect("Failed to execute pwd");
        assert!(pwd_result.success());
        assert_eq!(pwd_result.stdout.trim(), "/");
        
        println!("✅ All basic commands working: python, ls, pwd");
    }
}