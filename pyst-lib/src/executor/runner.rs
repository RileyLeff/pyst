use crate::{Config, ExitCode};
use anyhow::Result;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::env;

pub struct Executor {
    config: Config,
    // Runtime overrides from CLI
    pub offline_override: Option<bool>,
    pub cwd_override: Option<PathBuf>,
    pub uv_flags_override: Option<Vec<String>>,
}

impl Executor {
    pub fn new(config: Config) -> Self {
        Self { 
            config,
            offline_override: None,
            cwd_override: None,
            uv_flags_override: None,
        }
    }
    
    pub fn with_overrides(
        config: Config, 
        offline: Option<bool>,
        cwd: Option<PathBuf>,
        uv_flags: Option<Vec<String>>
    ) -> Self {
        Self {
            config,
            offline_override: offline,
            cwd_override: cwd,
            uv_flags_override: uv_flags,
        }
    }

    pub async fn run_script(
        &self,
        script_path: &PathBuf,
        args: &[String],
        _force: bool,
        dry_run: bool,
    ) -> Result<ExitCode> {
        let mut cmd = Command::new("uv");
        cmd.arg("run");
        
        // Add uv flags with precedence: CLI override > env > config
        let uv_flags = self.resolve_uv_flags();
        for flag in &uv_flags {
            cmd.arg(flag);
        }
        
        // Add script path
        cmd.arg(script_path);
        
        // Add separator between uv args and script args only if necessary
        // We need -- when script args might be confused with uv options
        if !uv_flags.is_empty() && !args.is_empty() && Self::needs_separator(&args) {
            cmd.arg("--");
        }
        
        // Add script arguments  
        for arg in args {
            cmd.arg(arg);
        }
        
        // Set working directory
        if let Some(cwd) = self.resolve_working_directory(script_path)? {
            cmd.current_dir(&cwd);
        }
        
        // Apply offline mode
        if self.is_offline_mode() {
            cmd.env("UV_NO_NETWORK", "1");
        }
        
        if dry_run {
            let cwd_info = if let Some(cwd) = self.resolve_working_directory(script_path)? {
                format!(" (cwd: {})", cwd.display())
            } else {
                String::new()
            };
            
            let uv_flags_info = if uv_flags.is_empty() {
                String::new()
            } else {
                format!(" [uv flags: {}]", uv_flags.join(" "))
            };
            
            let offline_info = if self.is_offline_mode() {
                " [UV_NO_NETWORK=1]"
            } else {
                ""
            };
            
            println!("Would execute: uv run{} {} -- {}{}{}{}", 
                if uv_flags.is_empty() { "" } else { &format!(" {}", uv_flags.join(" ")) },
                script_path.display(), 
                args.join(" "),
                cwd_info,
                uv_flags_info,
                offline_info
            );
            return Ok(ExitCode::Success);
        }
        
        // Stream stdio to allow real-time output
        let status = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        if status.success() {
            Ok(ExitCode::Success)
        } else {
            let exit_code = status.code().unwrap_or(1);
            if exit_code == 127 {
                Ok(ExitCode::ScriptNotFound)
            } else {
                Ok(ExitCode::GenericError)
            }
        }
    }
    
    fn resolve_uv_flags(&self) -> Vec<String> {
        // CLI override takes highest precedence
        if let Some(ref override_flags) = self.uv_flags_override {
            return override_flags.clone();
        }
        
        // Check environment variable
        if let Ok(env_flags) = env::var("PYST_UV_FLAGS") {
            return env_flags.split_whitespace().map(|s| s.to_string()).collect();
        }
        
        // Fall back to config
        self.config.core.uv.flags.clone()
    }
    
    fn resolve_working_directory(&self, script_path: &PathBuf) -> Result<Option<PathBuf>> {
        // CLI override takes precedence
        if let Some(ref override_cwd) = self.cwd_override {
            return Ok(Some(override_cwd.clone()));
        }
        
        match self.config.core.cwd.as_str() {
            "project" => {
                // Find project root
                if let Ok(current_dir) = env::current_dir() {
                    if let Some(project_root) = crate::discovery::Discovery::find_project_root(&current_dir) {
                        Ok(Some(project_root.path))
                    } else {
                        // No project root found, use current directory
                        Ok(Some(current_dir))
                    }
                } else {
                    Ok(None)
                }
            },
            "script" => {
                // Use script's parent directory
                Ok(script_path.parent().map(|p| p.to_path_buf()))
            },
            "current" => {
                // Use current working directory (no change needed)
                Ok(None)
            },
            custom_path => {
                // Use custom path from config
                let expanded = self.config.expand_path(custom_path)?;
                Ok(Some(expanded))
            }
        }
    }
    
    fn is_offline_mode(&self) -> bool {
        // CLI override takes precedence
        if let Some(offline) = self.offline_override {
            return offline;
        }
        
        // Check environment variable
        if let Ok(env_offline) = env::var("PYST_OFFLINE") {
            return env_offline.parse().unwrap_or(false);
        }
        
        // Fall back to config
        self.config.core.offline
    }
    
    /// Determine if we need a -- separator to avoid confusing script args with uv options
    fn needs_separator(args: &[String]) -> bool {
        // Check if any argument exactly matches uv options that could cause conflicts
        // We only need -- if script args would be interpreted as uv options
        args.iter().any(|arg| {
            matches!(arg.as_str(),
                // Python interpreter options
                "--python" | "-p" |
                // Index/package options  
                "--index" | "--default-index" | "--index-url" | "--extra-index-url" |
                "--find-links" | "-f" | "--no-index" |
                // Cache options
                "--cache-dir" | "--no-cache" | "-n" | "--refresh" |
                // Build options
                "--no-build" | "--no-binary" | "--no-build-isolation" |
                // Global options
                "--offline" | "--directory" | "--project" | "--config-file" | "--no-config" |
                "--help" | "-h" |
                // Only exact short flags that uv uses
                "-v" | "-q" | "-C" | "-U" | "-P" | "-w" | "-m" | "-s" | "-i"
            ) || arg.starts_with("--python=") || arg.starts_with("--index=") || 
                 arg.starts_with("--cache-dir=") || arg.starts_with("--directory=") ||
                 arg.starts_with("--project=") || arg.starts_with("--config-file=")
        })
    }
}