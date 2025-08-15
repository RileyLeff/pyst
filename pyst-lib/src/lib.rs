pub mod config;
pub mod discovery;
pub mod executor;
pub mod introspection;
pub mod install;
pub mod document;
pub mod mcp;

pub use config::Config;
pub use discovery::{Discovery, ProjectRoot, ScriptInfo, EntryPoint};
pub use executor::{Executor, ExitCode};
pub use install::{Installer, InstallSource, InstalledScript};
pub use document::Documenter;

use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Context {
    pub config: Config,
    pub project_root: Option<PathBuf>,
}

impl Context {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let project_root = Discovery::find_project_root(&std::env::current_dir()?)
            .map(|pr| pr.path);
        
        Ok(Self {
            config,
            project_root,
        })
    }
}