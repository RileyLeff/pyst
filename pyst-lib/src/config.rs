use crate::executor::context::Contexts;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    #[serde(default = "default_global_script_dirs")]
    pub global_script_dirs: Vec<String>,
    
    #[serde(default = "default_project_script_dir")]
    pub project_script_dir: String,
    
    #[serde(default = "default_precedence")]
    pub precedence: String,
    
    #[serde(default)]
    pub offline: bool,
    
    #[serde(default = "default_cwd")]
    pub cwd: String,
    
    #[serde(default = "default_introspection")]
    pub introspection: String,
    
    #[serde(default)]
    pub uv: UvConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UvConfig {
    #[serde(default)]
    pub flags: Vec<String>,
}

impl Default for UvConfig {
    fn default() -> Self {
        Self {
            flags: vec!["--python-preference=managed".to_string()],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    
    #[serde(default = "default_model")]
    pub model: String,
    
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
    
    #[serde(default)]
    pub redact: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    
    #[serde(default)]
    pub document: DocumentConfig,
    
    #[serde(default)]
    pub contexts: Contexts,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            document: DocumentConfig::default(),
            contexts: Contexts::default(),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            global_script_dirs: default_global_script_dirs(),
            project_script_dir: default_project_script_dir(),
            precedence: default_precedence(),
            offline: false,
            cwd: default_cwd(),
            introspection: default_introspection(),
            uv: UvConfig::default(),
        }
    }
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            api_key_env: default_api_key_env(),
            redact: vec!["SECRET_*".to_string(), "API_KEY_*".to_string()],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::load_cascading(None)
    }
    
    pub fn load_with_override(config_path: Option<PathBuf>) -> Result<Self> {
        Self::load_cascading(config_path)
    }
    
    fn load_cascading(override_path: Option<PathBuf>) -> Result<Self> {
        let mut config = Self::default();
        
        // 4. Built-in defaults (already applied via Default)
        
        // 3. Global config
        if let Some(global_config_path) = Self::get_global_config_path()? {
            if global_config_path.exists() {
                let global_config = Self::load_from_file(&global_config_path)?;
                config = config.merge_with(global_config)?;
            }
        }
        
        // 2. Project-local config
        if let Some(project_config_path) = Self::find_project_config()? {
            let project_config = Self::load_from_file(&project_config_path)?;
            config = config.merge_with(project_config)?;
        }
        
        // 1. Explicit override (highest precedence)
        if let Some(override_path) = override_path {
            if override_path.exists() {
                let override_config = Self::load_from_file(&override_path)?;
                config = config.merge_with(override_config)?;
            } else {
                return Err(anyhow!("Config file not found: {}", override_path.display()));
            }
        }
        
        // Environment variables (highest precedence)
        config = config.apply_env_overrides()?;
        
        Ok(config)
    }
    
    fn get_global_config_path() -> Result<Option<PathBuf>> {
        if let Some(config_dir) = dirs::config_dir() {
            let global_config = config_dir.join("pyst").join("pyst.toml");
            Ok(Some(global_config))
        } else {
            Ok(None)
        }
    }
    
    fn find_project_config() -> Result<Option<PathBuf>> {
        let current_dir = std::env::current_dir()?;
        let mut dir = current_dir.as_path();
        
        loop {
            let config_path = dir.join(".pyst.toml");
            if config_path.exists() {
                return Ok(Some(config_path));
            }
            
            if let Some(parent) = dir.parent() {
                dir = parent;
            } else {
                break;
            }
        }
        
        Ok(None)
    }
    
    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    fn merge_with(mut self, other: Self) -> Result<Self> {
        // Merge core config
        if !other.core.global_script_dirs.is_empty() {
            self.core.global_script_dirs = other.core.global_script_dirs;
        }
        if other.core.project_script_dir != default_project_script_dir() {
            self.core.project_script_dir = other.core.project_script_dir;
        }
        if other.core.precedence != default_precedence() {
            self.core.precedence = other.core.precedence;
        }
        if other.core.offline {
            self.core.offline = other.core.offline;
        }
        if other.core.cwd != default_cwd() {
            self.core.cwd = other.core.cwd;
        }
        if other.core.introspection != default_introspection() {
            self.core.introspection = other.core.introspection;
        }
        if !other.core.uv.flags.is_empty() {
            self.core.uv.flags = other.core.uv.flags;
        }
        
        // Merge document config
        if other.document.provider != default_provider() {
            self.document.provider = other.document.provider;
        }
        if other.document.model != default_model() {
            self.document.model = other.document.model;
        }
        if other.document.api_key_env != default_api_key_env() {
            self.document.api_key_env = other.document.api_key_env;
        }
        if !other.document.redact.is_empty() {
            self.document.redact = other.document.redact;
        }
        
        // Merge contexts (maps are merged recursively)
        for (context_name, context_config) in other.contexts.contexts {
            self.contexts.contexts.insert(context_name, context_config);
        }
        
        Ok(self)
    }
    
    fn apply_env_overrides(mut self) -> Result<Self> {
        // Check environment variables and override config values
        if let Ok(val) = std::env::var("PYST_OFFLINE") {
            self.core.offline = val.parse().unwrap_or(false);
        }
        
        if let Ok(val) = std::env::var("PYST_PRECEDENCE") {
            if val == "local" || val == "global" {
                self.core.precedence = val;
            }
        }
        
        if let Ok(val) = std::env::var("PYST_INTROSPECTION") {
            if val == "safe" || val == "import" {
                self.core.introspection = val;
            }
        }
        
        if let Ok(val) = std::env::var("PYST_PROJECT_SCRIPT_DIR") {
            self.core.project_script_dir = val;
        }
        
        if let Ok(val) = std::env::var("PYST_GLOBAL_SCRIPT_DIRS") {
            let dirs: Vec<String> = val.split(':').map(|s| s.to_string()).collect();
            if !dirs.is_empty() {
                self.core.global_script_dirs = dirs;
            }
        }
        
        Ok(self)
    }
    
    pub fn get_global_script_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        
        for dir_str in &self.core.global_script_dirs {
            let path = self.expand_path(dir_str)?;
            dirs.push(path);
        }
        
        Ok(dirs)
    }
    
    pub fn get_cache_dir(&self) -> Result<PathBuf> {
        if let Some(cache_dir) = dirs::cache_dir() {
            Ok(cache_dir.join("pyst"))
        } else {
            // Fallback to home directory
            if let Some(home) = dirs::home_dir() {
                Ok(home.join(".cache").join("pyst"))
            } else {
                Err(anyhow!("Cannot determine cache directory"))
            }
        }
    }
    
    pub fn get_data_dir(&self) -> Result<PathBuf> {
        if let Some(data_dir) = dirs::data_dir() {
            Ok(data_dir.join("pyst"))
        } else {
            // Fallback to home directory
            if let Some(home) = dirs::home_dir() {
                Ok(home.join(".local").join("share").join("pyst"))
            } else {
                Err(anyhow!("Cannot determine data directory"))
            }
        }
    }
    
    fn expand_path(&self, path_str: &str) -> Result<PathBuf> {
        if path_str.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                Ok(home.join(&path_str[2..]))
            } else {
                Err(anyhow!("Cannot expand ~: home directory not found"))
            }
        } else {
            Ok(PathBuf::from(path_str))
        }
    }
}

fn default_global_script_dirs() -> Vec<String> {
    vec!["~/.local/share/pyst/scripts".to_string()]
}

fn default_project_script_dir() -> String {
    ".pyst".to_string()
}

fn default_precedence() -> String {
    "local".to_string()
}

fn default_cwd() -> String {
    "project".to_string()
}

fn default_introspection() -> String {
    "safe".to_string()
}

fn default_provider() -> String {
    "gemini".to_string()
}

fn default_model() -> String {
    "gemini-1.5-flash".to_string()
}

fn default_api_key_env() -> String {
    "GOOGLE_API_KEY".to_string()
}