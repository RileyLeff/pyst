pub mod config;
pub mod discovery;
pub mod document;
pub mod executor;
pub mod install;
pub mod introspection;
pub mod mcp;

pub use config::Config;
pub use discovery::{Discovery, EntryPoint, ProjectRoot, ScriptInfo};
pub use document::Documenter;
pub use executor::{Executor, ExitCode};
pub use install::{InstallSource, InstalledScript, Installer};
pub use mcp::McpServer;

// Context and CliOverrides are defined in this module

use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Context {
    pub config: Config,
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub context: Option<String>,
    pub config_path: Option<PathBuf>,
    pub no_cache: bool,
    pub offline: Option<bool>,
    pub cwd: Option<PathBuf>,
    pub uv_flags: Option<Vec<String>>,
}

impl Context {
    pub fn new() -> Result<Self> {
        Self::new_with_overrides(None, CliOverrides::default())
    }

    pub fn new_with_overrides(
        config_override: Option<PathBuf>,
        cli_overrides: CliOverrides,
    ) -> Result<Self> {
        let config = Config::load_with_override(config_override)?;
        let project_root =
            Discovery::find_project_root(&std::env::current_dir()?).map(|pr| pr.path);

        // Apply context override by setting environment variable
        if let Some(context) = cli_overrides.context {
            std::env::set_var("PYST_CONTEXT", context);
        }

        Ok(Self {
            config,
            project_root,
        })
    }
}
