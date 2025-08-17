use crate::config::Config;
use crate::introspection::schema::{IntrospectionResult, SCHEMA_VERSION};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    pub file_hash: String,
    pub dependency_hash: String,
    pub python_version: String,
    pub schema_version: String,
    pub timestamp: u64,
    pub result: IntrospectionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheIndex {
    pub entries: HashMap<String, CacheEntry>,
    pub version: String,
}

pub struct Cache {
    cache_dir: PathBuf,
    index_path: PathBuf,
    index: CacheIndex,
}

impl Cache {
    pub fn new(config: &Config) -> Result<Self> {
        let cache_dir = config.get_cache_dir()?;
        let index_path = cache_dir.join("introspection_index.json");

        // Ensure cache directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        // Load or create index
        let index = if index_path.exists() {
            Self::load_index(&index_path)?
        } else {
            CacheIndex {
                entries: HashMap::new(),
                version: SCHEMA_VERSION.to_string(),
            }
        };

        Ok(Self {
            cache_dir,
            index_path,
            index,
        })
    }

    pub fn get(&self, script_path: &Path) -> Option<&IntrospectionResult> {
        let key = self.make_cache_key(script_path);

        if let Some(entry) = self.index.entries.get(&key) {
            // Check if cache entry is still valid
            if self.is_cache_valid(script_path, entry).unwrap_or(false) {
                return Some(&entry.result);
            }
        }

        None
    }

    pub fn put(&mut self, script_path: &Path, result: IntrospectionResult) -> Result<()> {
        let key = self.make_cache_key(script_path);
        let file_hash = Self::hash_file(script_path)?;
        let dependency_hash = self.hash_dependencies(script_path)?;
        let python_version = Self::get_python_version()?;

        let entry = CacheEntry {
            file_hash,
            dependency_hash,
            python_version,
            schema_version: SCHEMA_VERSION.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            result,
        };

        self.index.entries.insert(key, entry);
        self.save_index()?;

        Ok(())
    }

    pub fn invalidate(&mut self, script_path: &Path) -> Result<()> {
        let key = self.make_cache_key(script_path);
        self.index.entries.remove(&key);
        self.save_index()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.index.entries.clear();
        self.save_index()?;

        // Remove all cache files
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
        }

        Ok(())
    }

    pub fn get_cache_path(&self) -> &Path {
        &self.cache_dir
    }

    pub fn get_stats(&self) -> CacheStats {
        let total_entries = self.index.entries.len();
        let valid_entries = self
            .index
            .entries
            .iter()
            .filter(|(key, entry)| {
                if let Ok(script_path) = self.key_to_path(key) {
                    self.is_cache_valid(&script_path, entry).unwrap_or(false)
                } else {
                    false
                }
            })
            .count();

        CacheStats {
            total_entries,
            valid_entries,
            invalid_entries: total_entries - valid_entries,
        }
    }

    fn make_cache_key(&self, script_path: &Path) -> String {
        let mut hasher = Sha256::new();
        hasher.update(script_path.to_string_lossy().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn hash_file(path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn hash_dependencies(&self, script_path: &Path) -> Result<String> {
        // Hash content of potential dependency files
        let mut hasher = Sha256::new();

        // Check for requirements.txt, pyproject.toml, etc.
        if let Some(parent) = script_path.parent() {
            let requirements_txt = parent.join("requirements.txt");
            if requirements_txt.exists() {
                if let Ok(content) = fs::read(&requirements_txt) {
                    hasher.update(&content);
                }
            }

            let pyproject_toml = parent.join("pyproject.toml");
            if pyproject_toml.exists() {
                if let Ok(content) = fs::read(&pyproject_toml) {
                    hasher.update(&content);
                }
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn get_python_version() -> Result<String> {
        // Use uv run to match the environment that introspection actually uses
        let output = std::process::Command::new("uv")
            .args(["run", "python", "--version"])
            .output()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            // Fallback to system python3 if uv is not available
            let output = std::process::Command::new("python3")
                .args(["--version"])
                .output()?;

            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                Ok(version.trim().to_string())
            } else {
                Err(anyhow!("Failed to get Python version"))
            }
        }
    }

    fn is_cache_valid(&self, script_path: &Path, entry: &CacheEntry) -> Result<bool> {
        // Check if file still exists
        if !script_path.exists() {
            return Ok(false);
        }

        // Check if schema version matches
        if entry.schema_version != SCHEMA_VERSION {
            return Ok(false);
        }

        // Check if file content has changed
        let current_file_hash = Self::hash_file(script_path)?;
        if entry.file_hash != current_file_hash {
            return Ok(false);
        }

        // Check if dependencies have changed
        let current_dep_hash = self.hash_dependencies(script_path)?;
        if entry.dependency_hash != current_dep_hash {
            return Ok(false);
        }

        // Check if Python version has changed
        let current_python_version = Self::get_python_version()?;
        if entry.python_version != current_python_version {
            return Ok(false);
        }

        Ok(true)
    }

    fn key_to_path(&self, _key: &str) -> Result<PathBuf> {
        // This is a simplified version - in practice we'd need to store
        // the path in the cache entry or use a different approach
        Err(anyhow!("Cannot reverse engineer path from key"))
    }

    fn load_index(path: &Path) -> Result<CacheIndex> {
        let content = fs::read_to_string(path)?;
        let index: CacheIndex = serde_json::from_str(&content)?;
        Ok(index)
    }

    fn save_index(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.index)?;
        fs::write(&self.index_path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub invalid_entries: usize,
}
