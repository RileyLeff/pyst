use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledScript {
    pub name: String,
    pub source: InstallSource,
    pub install_path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub commit_sha: Option<String>,
    pub file_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallSource {
    GitHubRepo { 
        owner: String, 
        repo: String, 
        path: Option<String> 
    },
    GitHubGist { 
        gist_id: String, 
        file: Option<String> 
    },
    RawUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallManifest {
    pub version: String,
    pub scripts: HashMap<String, InstalledScript>,
}

impl InstallManifest {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            scripts: HashMap::new(),
        }
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        let manifest: InstallManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

pub struct Installer {
    manifest_path: PathBuf,
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        let manifest_path = install_dir.join("manifest.json");
        Self { manifest_path, install_dir }
    }
    
    pub fn parse_source(source: &str) -> Result<InstallSource> {
        let url = Url::parse(source)?;
        
        match url.host_str() {
            Some("github.com") => Self::parse_github_url(&url),
            Some("gist.github.com") => Self::parse_gist_url(&url),
            _ => Ok(InstallSource::RawUrl { url: source.to_string() }),
        }
    }
    
    fn parse_github_url(url: &Url) -> Result<InstallSource> {
        let path_segments: Vec<&str> = url.path_segments()
            .ok_or_else(|| anyhow!("Invalid GitHub URL"))?
            .collect();
            
        if path_segments.len() < 2 {
            return Err(anyhow!("Invalid GitHub URL: missing owner/repo"));
        }
        
        let owner = path_segments[0].to_string();
        let repo = path_segments[1].to_string();
        
        // Check if this is a specific file path
        let file_path = if path_segments.len() > 4 && path_segments[2] == "blob" {
            // Skip "blob" and branch name, take the rest as file path
            Some(path_segments[4..].join("/"))
        } else {
            None
        };
        
        Ok(InstallSource::GitHubRepo {
            owner,
            repo,
            path: file_path,
        })
    }
    
    fn parse_gist_url(url: &Url) -> Result<InstallSource> {
        let path_segments: Vec<&str> = url.path_segments()
            .ok_or_else(|| anyhow!("Invalid Gist URL"))?
            .collect();
            
        if path_segments.is_empty() {
            return Err(anyhow!("Invalid Gist URL: missing gist ID"));
        }
        
        let gist_id = path_segments[0].to_string();
        let file = if path_segments.len() > 1 {
            Some(path_segments[1].to_string())
        } else {
            None
        };
        
        Ok(InstallSource::GitHubGist { gist_id, file })
    }
    
    pub async fn install(&self, source: &str, name: Option<&str>) -> Result<Vec<String>> {
        let install_source = Self::parse_source(source)?;
        
        match install_source {
            InstallSource::GitHubRepo { owner, repo, path } => {
                self.install_from_github(&owner, &repo, path.as_deref(), name).await
            }
            InstallSource::GitHubGist { gist_id, file } => {
                self.install_from_gist(&gist_id, file.as_deref(), name).await
            }
            InstallSource::RawUrl { url } => {
                self.install_from_url(&url, name).await
            }
        }
    }
    
    async fn install_from_github(&self, owner: &str, repo: &str, file_path: Option<&str>, name: Option<&str>) -> Result<Vec<String>> {
        let temp_dir = tempfile::tempdir()?;
        let repo_url = format!("https://github.com/{}/{}.git", owner, repo);
        
        // Clone the repository
        let git_repo = git2::Repository::clone(&repo_url, temp_dir.path())?;
        
        // Get the current commit SHA for pinning
        let head = git_repo.head()?;
        let commit = head.peel_to_commit()?;
        let commit_sha = commit.id().to_string();
        
        let mut installed_scripts = Vec::new();
        
        if let Some(specific_file) = file_path {
            // Install specific file
            let file_path = temp_dir.path().join(specific_file);
            if file_path.exists() && file_path.extension().and_then(|s| s.to_str()) == Some("py") {
                let script_name = name.unwrap_or(
                    &file_path.file_stem().unwrap().to_string_lossy()
                ).to_string();
                
                self.install_script_file(&file_path, &script_name, InstallSource::GitHubRepo {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    path: Some(specific_file.to_string()),
                }, Some(commit_sha.clone())).await?;
                
                installed_scripts.push(script_name);
            }
        } else {
            // Install all Python files in the repo
            for entry in walkdir::WalkDir::new(temp_dir.path()) {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() 
                    && path.extension().and_then(|s| s.to_str()) == Some("py")
                    && !path.file_name().unwrap().to_string_lossy().starts_with('_')
                    && !path.file_name().unwrap().to_string_lossy().starts_with('.')
                {
                    let script_name = path.file_stem().unwrap().to_string_lossy().to_string();
                    
                    self.install_script_file(path, &script_name, InstallSource::GitHubRepo {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                        path: None,
                    }, Some(commit_sha.clone())).await?;
                    
                    installed_scripts.push(script_name);
                }
            }
        }
        
        Ok(installed_scripts)
    }
    
    async fn install_from_gist(&self, gist_id: &str, file: Option<&str>, name: Option<&str>) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        let gist_api_url = format!("https://api.github.com/gists/{}", gist_id);
        
        let response = client.get(&gist_api_url)
            .header("User-Agent", "pyst")
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch gist: {}", response.status()));
        }
        
        let gist_data: serde_json::Value = response.json().await?;
        let files = gist_data["files"].as_object()
            .ok_or_else(|| anyhow!("Invalid gist response: no files"))?;
        
        let mut installed_scripts = Vec::new();
        
        for (filename, file_data) in files {
            if let Some(target_file) = file {
                if filename != target_file {
                    continue;
                }
            }
            
            if !filename.ends_with(".py") {
                continue;
            }
            
            let content = file_data["content"].as_str()
                .ok_or_else(|| anyhow!("No content in gist file"))?;
            
            let script_name = name.unwrap_or(
                &filename.strip_suffix(".py").unwrap_or(filename)
            ).to_string();
            
            let temp_file = tempfile::NamedTempFile::new()?;
            std::fs::write(temp_file.path(), content)?;
            
            self.install_script_file(temp_file.path(), &script_name, InstallSource::GitHubGist {
                gist_id: gist_id.to_string(),
                file: Some(filename.clone()),
            }, None).await?;
            
            installed_scripts.push(script_name);
        }
        
        Ok(installed_scripts)
    }
    
    async fn install_from_url(&self, url: &str, name: Option<&str>) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download file: {}", response.status()));
        }
        
        let content = response.text().await?;
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(temp_file.path(), content)?;
        
        let script_name = if let Some(name) = name {
            name.to_string()
        } else {
            // Extract filename from URL
            let parsed_url = Url::parse(url)?;
            let url_path = parsed_url.path();
            let filename = url_path.split('/').last().unwrap_or("script");
            filename.strip_suffix(".py").unwrap_or(filename).to_string()
        };
        
        self.install_script_file(temp_file.path(), &script_name, InstallSource::RawUrl {
            url: url.to_string(),
        }, None).await?;
        
        Ok(vec![script_name])
    }
    
    async fn install_script_file(&self, source_path: &Path, script_name: &str, source: InstallSource, commit_sha: Option<String>) -> Result<()> {
        // Create install directory
        std::fs::create_dir_all(&self.install_dir)?;
        
        // Read and hash the file content
        let content = std::fs::read(source_path)?;
        let file_hash = hex::encode(sha2::Sha256::digest(content));
        
        // Install path
        let install_path = self.install_dir.join(format!("{}.py", script_name));
        
        // Copy file to install location
        std::fs::copy(source_path, &install_path)?;
        
        // Load and update manifest
        let mut manifest = InstallManifest::load_from_file(&self.manifest_path)?;
        
        let installed_script = InstalledScript {
            name: script_name.to_string(),
            source,
            install_path: install_path.clone(),
            installed_at: chrono::Utc::now(),
            commit_sha,
            file_hash,
        };
        
        manifest.scripts.insert(script_name.to_string(), installed_script);
        manifest.save_to_file(&self.manifest_path)?;
        
        Ok(())
    }
    
    pub async fn uninstall(&self, script: &str) -> Result<()> {
        let mut manifest = InstallManifest::load_from_file(&self.manifest_path)?;
        
        let installed_script = manifest.scripts.remove(script)
            .ok_or_else(|| anyhow!("Script '{}' is not installed", script))?;
        
        // Remove the script file
        if installed_script.install_path.exists() {
            std::fs::remove_file(&installed_script.install_path)?;
        }
        
        // Save updated manifest
        manifest.save_to_file(&self.manifest_path)?;
        
        Ok(())
    }
    
    pub async fn update(&self, script: &str) -> Result<()> {
        let manifest = InstallManifest::load_from_file(&self.manifest_path)?;
        
        let installed_script = manifest.scripts.get(script)
            .ok_or_else(|| anyhow!("Script '{}' is not installed", script))?;
        
        // Uninstall current version
        self.uninstall(script).await?;
        
        // Reinstall from the same source
        match &installed_script.source {
            InstallSource::GitHubRepo { owner, repo, path } => {
                let source_url = if let Some(path) = path {
                    format!("https://github.com/{}/{}/blob/main/{}", owner, repo, path)
                } else {
                    format!("https://github.com/{}/{}", owner, repo)
                };
                self.install(&source_url, Some(script)).await?;
            }
            InstallSource::GitHubGist { gist_id, .. } => {
                let source_url = format!("https://gist.github.com/{}", gist_id);
                self.install(&source_url, Some(script)).await?;
            }
            InstallSource::RawUrl { url } => {
                self.install(url, Some(script)).await?;
            }
        }
        
        Ok(())
    }
    
    pub fn list_installed(&self) -> Result<Vec<InstalledScript>> {
        let manifest = InstallManifest::load_from_file(&self.manifest_path)?;
        Ok(manifest.scripts.into_values().collect())
    }
}