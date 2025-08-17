use crate::config::Config;
use crate::introspection::cache::{Cache, CacheStats};
use crate::introspection::schema::IntrospectionResult;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// Embed the introspector script at compile time
const INTROSPECTOR_SRC: &str = include_str!("../../helpers/introspector.py");

fn ensure_introspector_installed(config: &Config) -> Result<PathBuf> {
    let helpers_dir = config.get_data_dir()?.join("helpers");
    fs::create_dir_all(&helpers_dir)?;
    let target = helpers_dir.join("introspector.py");

    // Write only if missing or stale (for now, just check if exists)
    if !target.exists() {
        let mut file = fs::File::create(&target)?;
        file.write_all(INTROSPECTOR_SRC.as_bytes())?;
    }
    Ok(target)
}

pub struct IntrospectionRunner {
    config: Config,
    cache: Cache,
    trusted_paths: HashSet<PathBuf>,
    no_cache: bool,
    offline_override: Option<bool>,
}

impl IntrospectionRunner {
    pub fn new(config: Config) -> Result<Self> {
        let cache = Cache::new(&config)?;
        let trusted_paths = Self::load_trusted_paths(&config)?;

        Ok(Self {
            config,
            cache,
            trusted_paths,
            no_cache: false,
            offline_override: None,
        })
    }

    pub fn new_with_no_cache(config: Config, no_cache: bool) -> Result<Self> {
        let cache = Cache::new(&config)?;
        let trusted_paths = Self::load_trusted_paths(&config)?;

        Ok(Self {
            config,
            cache,
            trusted_paths,
            no_cache,
            offline_override: None,
        })
    }

    pub fn new_with_overrides(
        config: Config,
        no_cache: bool,
        offline_override: Option<bool>,
    ) -> Result<Self> {
        let cache = Cache::new(&config)?;
        let trusted_paths = Self::load_trusted_paths(&config)?;

        Ok(Self {
            config,
            cache,
            trusted_paths,
            no_cache,
            offline_override,
        })
    }

    pub fn introspect(&mut self, script_path: &Path) -> Result<IntrospectionResult> {
        // Check cache first (unless no-cache is enabled)
        if !self.no_cache {
            if let Some(cached_result) = self.cache.get(script_path) {
                return Ok(cached_result.clone());
            }
        }

        // Determine introspection mode
        let mode = self.determine_mode(script_path);

        // Run introspection
        let result = self.run_introspection(script_path, mode)?;

        // Cache the result (unless no-cache is enabled)
        if !self.no_cache {
            if let Err(e) = self.cache.put(script_path, result.clone()) {
                eprintln!("Warning: Failed to cache introspection result: {}", e);
            }
        }

        Ok(result)
    }

    pub fn introspect_batch(
        &mut self,
        script_paths: &[PathBuf],
    ) -> Result<Vec<IntrospectionResult>> {
        // If only one script or few scripts, use individual introspection to hit cache
        if script_paths.len() <= 3 {
            let mut results = Vec::new();
            for script_path in script_paths {
                match self.introspect(script_path) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to introspect {}: {}",
                            script_path.display(),
                            e
                        );
                        // Create minimal error result
                        results.push(IntrospectionResult {
                            schema_version: crate::introspection::schema::SCHEMA_VERSION.to_string(),
                            python_version: "Unknown".to_string(),
                            script_hash: "error".to_string(),
                            metadata: Default::default(),
                        });
                    }
                }
            }
            return Ok(results);
        }

        // For larger batches, try the optimized batch mode first
        match self.run_batch_introspection(script_paths) {
            Ok(results) => Ok(results),
            Err(e) => {
                eprintln!("Warning: Batch introspection failed ({}), falling back to individual processing", e);
                // Fall back to individual processing
                let mut results = Vec::new();
                for script_path in script_paths {
                    match self.introspect(script_path) {
                        Ok(result) => results.push(result),
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to introspect {}: {}",
                                script_path.display(),
                                e
                            );
                            // Create minimal error result
                            results.push(IntrospectionResult {
                                schema_version: crate::introspection::schema::SCHEMA_VERSION.to_string(),
                                python_version: "Unknown".to_string(),
                                script_hash: "error".to_string(),
                                metadata: Default::default(),
                            });
                        }
                    }
                }
                Ok(results)
            }
        }
    }

    pub fn invalidate_cache(&mut self, script_path: &Path) -> Result<()> {
        self.cache.invalidate(script_path)
    }

    pub fn clear_cache(&mut self) -> Result<()> {
        self.cache.clear()
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats()
    }

    pub fn get_cache_path(&self) -> &Path {
        self.cache.get_cache_path()
    }

    pub fn trust_path(&mut self, path: &Path) -> Result<()> {
        let canonical_path = path.canonicalize()?;
        self.trusted_paths.insert(canonical_path.clone());
        self.save_trusted_paths()?;
        Ok(())
    }

    pub fn is_trusted(&self, script_path: &Path) -> bool {
        if let Ok(canonical_path) = script_path.canonicalize() {
            // Check if script itself is trusted
            if self.trusted_paths.contains(&canonical_path) {
                return true;
            }

            // Check if any parent directory is trusted
            for trusted_path in &self.trusted_paths {
                if canonical_path.starts_with(trusted_path) {
                    return true;
                }
            }
        }

        false
    }

    fn determine_mode(&self, script_path: &Path) -> IntrospectionMode {
        match self.config.core.introspection.as_str() {
            "import" if self.is_trusted(script_path) => IntrospectionMode::Import,
            "import" => {
                // Config says import but path is not trusted, use safe mode
                IntrospectionMode::Safe
            }
            _ => IntrospectionMode::Safe,
        }
    }

    fn run_introspection(
        &self,
        script_path: &Path,
        mode: IntrospectionMode,
    ) -> Result<IntrospectionResult> {
        let introspector_path = self.get_introspector_path()?;

        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg(&introspector_path)
            .arg(script_path)
            .arg("--mode")
            .arg(mode.to_string());

        // Apply offline mode if configured or overridden
        let is_offline = self.offline_override.unwrap_or(self.config.core.offline);
        if is_offline {
            cmd.env("UV_NO_NETWORK", "1");
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Introspection failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: IntrospectionResult = serde_json::from_str(&stdout)?;

        Ok(result)
    }

    fn run_batch_introspection(
        &self,
        script_paths: &[PathBuf],
    ) -> Result<Vec<IntrospectionResult>> {
        let introspector_path = self.get_introspector_path()?;

        // Serialize script paths to JSON
        let paths_json: Vec<String> = script_paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let batch_json = serde_json::to_string(&paths_json)?;

        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg(&introspector_path)
            .arg("--batch")
            .arg(&batch_json)
            .arg("--mode")
            .arg("safe"); // For now, always use safe mode in batch

        // Apply offline mode if configured or overridden
        let is_offline = self.offline_override.unwrap_or(self.config.core.offline);
        if is_offline {
            cmd.env("UV_NO_NETWORK", "1");
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Batch introspection failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results: Vec<IntrospectionResult> = serde_json::from_str(&stdout)?;

        Ok(results)
    }

    fn get_introspector_path(&self) -> Result<PathBuf> {
        // 1) Use embedded helper at the platform data dir
        if let Ok(path) = ensure_introspector_installed(&self.config) {
            return Ok(path);
        }

        // 2) Dev fallbacks (workspace-relative) remain as last resort
        let current_exe = std::env::current_exe()?;
        let exe_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow!("Cannot find executable directory"))?;

        let possible_paths = [
            exe_dir.join("../pyst-lib/helpers/introspector.py"),
            exe_dir.join("helpers/introspector.py"),
            PathBuf::from("pyst-lib/helpers/introspector.py"),
        ];

        for path in &possible_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        Err(anyhow!("Cannot find introspector.py helper script"))
    }

    fn load_trusted_paths(config: &Config) -> Result<HashSet<PathBuf>> {
        let data_dir = config.get_data_dir()?;
        let trusted_file = data_dir.join("trusted_paths.json");

        if trusted_file.exists() {
            let content = fs::read_to_string(&trusted_file)?;
            let paths: Vec<PathBuf> = serde_json::from_str(&content)?;
            Ok(paths.into_iter().collect())
        } else {
            Ok(HashSet::new())
        }
    }

    fn save_trusted_paths(&self) -> Result<()> {
        let data_dir = self.config.get_data_dir()?;
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let trusted_file = data_dir.join("trusted_paths.json");
        let paths: Vec<&PathBuf> = self.trusted_paths.iter().collect();
        let content = serde_json::to_string_pretty(&paths)?;
        fs::write(&trusted_file, content)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum IntrospectionMode {
    Safe,
    Import,
}

impl std::fmt::Display for IntrospectionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IntrospectionMode::Safe => write!(f, "safe"),
            IntrospectionMode::Import => write!(f, "import"),
        }
    }
}
