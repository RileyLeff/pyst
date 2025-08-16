use crate::Config;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Discovery {
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_local: bool,
    pub description: Option<String>,
    pub entry_point: EntryPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryPoint {
    Pep723,
    Framework(String),
    MainFunction,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ProjectRoot {
    pub path: PathBuf,
    pub source: ProjectRootSource,
}

#[derive(Debug, Clone)]
pub enum ProjectRootSource {
    PystToml,
    Git,
    CurrentDir,
}

impl Discovery {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn find_project_root(start_dir: &Path) -> Option<ProjectRoot> {
        let mut current = start_dir;

        loop {
            // Check for .pyst.toml
            if current.join(".pyst.toml").exists() {
                return Some(ProjectRoot {
                    path: current.to_path_buf(),
                    source: ProjectRootSource::PystToml,
                });
            }

            // Check for .git
            if current.join(".git").exists() {
                return Some(ProjectRoot {
                    path: current.to_path_buf(),
                    source: ProjectRootSource::Git,
                });
            }

            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }

        // Fallback to current directory
        Some(ProjectRoot {
            path: start_dir.to_path_buf(),
            source: ProjectRootSource::CurrentDir,
        })
    }

    pub fn discover_scripts(&self, project_root: Option<&Path>) -> Result<Vec<ScriptInfo>> {
        let mut scripts = Vec::new();

        // Discover local scripts if we have a project root
        if let Some(root) = project_root {
            let local_dir = root.join(&self.config.core.project_script_dir);
            if local_dir.exists() {
                scripts.extend(self.discover_scripts_in_dir(&local_dir, true)?);
            }
        }

        // Discover global scripts
        for global_dir in self.config.get_global_script_dirs()? {
            if global_dir.exists() {
                scripts.extend(self.discover_scripts_in_dir(&global_dir, false)?);
            }
        }

        Ok(scripts)
    }

    fn discover_scripts_in_dir(&self, dir: &Path, is_local: bool) -> Result<Vec<ScriptInfo>> {
        let mut scripts = Vec::new();

        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension() {
                if ext != "py" {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(file_name) = path.file_name() {
                let name_str = file_name.to_string_lossy();

                // Skip private files and type stubs
                if name_str.starts_with('_') || name_str.ends_with(".pyi") {
                    continue;
                }
            }

            if let Ok(script_info) = self.analyze_script(path, is_local) {
                scripts.push(script_info);
            }
        }

        Ok(scripts)
    }

    fn analyze_script(&self, path: &Path, is_local: bool) -> Result<ScriptInfo> {
        let content = std::fs::read_to_string(path)?;

        let name = path
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_string_lossy()
            .to_string();

        let entry_point = self.detect_entry_point(&content);

        Ok(ScriptInfo {
            name,
            path: path.to_path_buf(),
            is_local,
            description: None, // TODO: Extract from docstrings or metadata
            entry_point,
        })
    }

    fn detect_entry_point(&self, content: &str) -> EntryPoint {
        // Check for PEP 723 script metadata
        if content.contains("# /// script") {
            return EntryPoint::Pep723;
        }

        // Check for common framework patterns
        if content.contains("typer.") || content.contains("from typer") {
            return EntryPoint::Framework("typer".to_string());
        }

        if content.contains("click.") || content.contains("from click") {
            return EntryPoint::Framework("click".to_string());
        }

        // Check for main function
        if content.contains("def main(") {
            return EntryPoint::MainFunction;
        }

        EntryPoint::Unknown
    }

    pub fn resolve_script(&self, name: &str, project_root: Option<&Path>) -> Result<ScriptInfo> {
        // Handle explicit selectors
        if let Some((scope, script_name)) = name.split_once(':') {
            return match scope {
                "project" => self.resolve_local_script(script_name, project_root),
                "global" => self.resolve_global_script(script_name),
                _ => Err(anyhow!("Invalid scope: {}", scope)),
            };
        }

        // Handle direct path
        if name.contains('/') || name.contains('\\') {
            let path = PathBuf::from(name);
            if path.exists() {
                return self.analyze_script(&path, false);
            }
        }

        // Regular name resolution based on precedence
        let scripts = self.discover_scripts(project_root)?;

        if self.config.core.precedence == "local" {
            // Try local first, then global
            if let Some(script) = scripts.iter().find(|s| s.is_local && s.name == name) {
                return Ok(script.clone());
            }
            if let Some(script) = scripts.iter().find(|s| !s.is_local && s.name == name) {
                return Ok(script.clone());
            }
        } else {
            // Try global first, then local
            if let Some(script) = scripts.iter().find(|s| !s.is_local && s.name == name) {
                return Ok(script.clone());
            }
            if let Some(script) = scripts.iter().find(|s| s.is_local && s.name == name) {
                return Ok(script.clone());
            }
        }

        Err(anyhow!("Script not found: {}", name))
    }

    fn resolve_local_script(&self, name: &str, project_root: Option<&Path>) -> Result<ScriptInfo> {
        let root = project_root.ok_or_else(|| anyhow!("No project root found"))?;
        let local_dir = root.join(&self.config.core.project_script_dir);
        let scripts = self.discover_scripts_in_dir(&local_dir, true)?;

        scripts
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow!("Local script not found: {}", name))
    }

    fn resolve_global_script(&self, name: &str) -> Result<ScriptInfo> {
        for global_dir in self.config.get_global_script_dirs()? {
            if let Ok(scripts) = self.discover_scripts_in_dir(&global_dir, false) {
                if let Some(script) = scripts.into_iter().find(|s| s.name == name) {
                    return Ok(script);
                }
            }
        }

        Err(anyhow!("Global script not found: {}", name))
    }
}
