use std::path::Path;
use testcontainers::{core::ExecCommand, core::Mount, ContainerRequest, GenericImage, ImageExt};

/// Helper functions for creating standardized Pyst test containers
pub struct PystContainer;

impl PystContainer {
    /// Create a new Pyst test container with Python 3.11 (legacy)
    pub fn python_base() -> ContainerRequest<GenericImage> {
        GenericImage::new("python", "3.11-slim").with_cmd(vec!["sleep", "infinity"])
    }

    /// Create a Python container with basic environment setup for pyst testing (legacy)
    pub fn python_with_env() -> ContainerRequest<GenericImage> {
        GenericImage::new("python", "3.11-slim")
            .with_cmd(vec!["sleep", "infinity"])
            .with_env_var("PYTHONUNBUFFERED", "1")
            .with_env_var("DEBIAN_FRONTEND", "noninteractive")
            .with_env_var("UV_NO_NETWORK", "0") // Allow network for uv package installs
    }

    /// Create an optimized pyst development container with all tools pre-installed
    /// Uses the custom pyst-test:latest image built from our Dockerfile
    pub fn pyst_dev_base() -> ContainerRequest<GenericImage> {
        GenericImage::new("pyst-test", "latest").with_cmd(vec!["sleep", "infinity"])
    }

    /// Create a pyst container with project source mounted
    pub fn pyst_with_source<P: AsRef<Path>>(project_root: P) -> ContainerRequest<GenericImage> {
        let mount = Mount::bind_mount(
            project_root.as_ref().to_string_lossy().to_string(),
            "/workspace",
        );

        Self::pyst_dev_base()
            .with_mount(mount)
            .with_working_dir("/workspace")
    }

    /// Create a pyst container with test fixtures mounted
    pub fn pyst_with_fixtures<P: AsRef<Path>>(fixtures_dir: P) -> ContainerRequest<GenericImage> {
        let mount = Mount::bind_mount(
            fixtures_dir.as_ref().to_string_lossy().to_string(),
            "/test-fixtures",
        );

        Self::pyst_dev_base()
            .with_mount(mount)
            .with_working_dir("/test-fixtures")
    }

    /// Create an optimized pyst container with test fixtures mounted
    /// Uses the pre-built image with all tools already installed
    pub fn optimized_with_fixtures<P: AsRef<Path>>(
        fixtures_dir: P,
    ) -> ContainerRequest<GenericImage> {
        let mount = Mount::bind_mount(
            fixtures_dir.as_ref().to_string_lossy().to_string(),
            "/test-fixtures",
        );

        Self::pyst_dev_base()
            .with_mount(mount)
            .with_working_dir("/test-fixtures")
    }

    /// Create an optimized container for full development testing
    /// Uses the pre-built image and mounts both source and fixtures
    pub fn optimized_full_dev<P: AsRef<Path>, Q: AsRef<Path>>(
        project_root: P,
        fixtures_dir: Q,
    ) -> ContainerRequest<GenericImage> {
        let source_mount = Mount::bind_mount(
            project_root.as_ref().to_string_lossy().to_string(),
            "/workspace",
        );
        let fixtures_mount = Mount::bind_mount(
            fixtures_dir.as_ref().to_string_lossy().to_string(),
            "/test-project",
        );

        Self::pyst_dev_base()
            .with_mount(source_mount)
            .with_mount(fixtures_mount)
            .with_working_dir("/test-project")
    }
}

/// Helper for setting up pyst development environment in containers
pub struct PystSetup;

impl PystSetup {
    /// Install uv in the container
    pub async fn install_uv(
        container: &testcontainers::ContainerAsync<GenericImage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let result = CommandResult::exec(
            container,
            vec![
                "sh",
                "-c",
                "curl -LsSf https://astral.sh/uv/install.sh | sh",
            ],
        )
        .await?;

        if !result.success() {
            return Err(format!("Failed to install uv: {}", result.stderr).into());
        }

        Ok(())
    }

    /// Install Rust in the container  
    pub async fn install_rust(
        container: &testcontainers::ContainerAsync<GenericImage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Install curl and build-essential first
        let deps_result = CommandResult::exec(
            container,
            vec![
                "sh",
                "-c",
                "apt-get update && apt-get install -y curl build-essential git",
            ],
        )
        .await?;

        if !deps_result.success() {
            return Err(format!("Failed to install dependencies: {}", deps_result.stderr).into());
        }

        // Install Rust
        let rust_result = CommandResult::exec(
            container,
            vec![
                "sh",
                "-c",
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
            ],
        )
        .await?;

        if !rust_result.success() {
            return Err(format!("Failed to install Rust: {}", rust_result.stderr).into());
        }

        Ok(())
    }

    /// Build pyst from source in the container
    pub async fn build_pyst(
        container: &testcontainers::ContainerAsync<GenericImage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Source Rust environment and build
        let result = CommandResult::exec(
            container,
            vec![
                "sh",
                "-c",
                "source ~/.cargo/env && cd /workspace && cargo build --release",
            ],
        )
        .await?;

        if !result.success() {
            return Err(format!("Failed to build pyst: {}", result.stderr).into());
        }

        Ok("/workspace/target/release/pyst".to_string())
    }

    /// Complete setup: install tools and build pyst
    pub async fn full_setup(
        container: &testcontainers::ContainerAsync<GenericImage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Self::install_rust(container).await?;
        Self::install_uv(container).await?;
        Self::build_pyst(container).await
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
        cmd: Vec<&str>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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
        let result = CommandResult::exec(&container, vec!["python", "--version"])
            .await
            .expect("Failed to execute command");

        assert!(result.success(), "Python version command should succeed");
        assert!(
            result.stdout.contains("Python 3.11"),
            "Should return Python 3.11 version"
        );

        println!("✅ Basic Python container: {}", result.stdout.trim());
    }

    #[tokio::test]
    async fn test_container_with_env() {
        let image = PystContainer::python_with_env();
        let container = image.start().await.expect("Failed to start container");

        // Test that environment variables are set
        let result = CommandResult::exec(&container, vec!["env"])
            .await
            .expect("Failed to execute env command");

        assert!(result.success(), "Env command should succeed");
        assert!(
            result.stdout.contains("PYTHONUNBUFFERED=1"),
            "PYTHONUNBUFFERED should be set"
        );
        assert!(
            result.stdout.contains("DEBIAN_FRONTEND=noninteractive"),
            "DEBIAN_FRONTEND should be set"
        );

        println!("✅ Environment variables properly configured");
    }

    #[tokio::test]
    async fn test_basic_commands() {
        let image = PystContainer::python_base();
        let container = image.start().await.expect("Failed to start container");

        // Test python --version
        let py_result = CommandResult::exec(&container, vec!["python", "--version"])
            .await
            .expect("Failed to execute python --version");
        assert!(py_result.success());
        assert!(py_result.stdout.contains("Python"));

        // Test ls
        let ls_result = CommandResult::exec(&container, vec!["ls", "-la", "/"])
            .await
            .expect("Failed to execute ls");
        assert!(ls_result.success());
        assert!(ls_result.stdout.contains("bin"));

        // Test pwd
        let pwd_result = CommandResult::exec(&container, vec!["pwd"])
            .await
            .expect("Failed to execute pwd");
        assert!(pwd_result.success());
        assert_eq!(pwd_result.stdout.trim(), "/");

        println!("✅ All basic commands working: python, ls, pwd");
    }
}
