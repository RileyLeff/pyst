use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use pyst_lib::{Context, Discovery, Executor, ExitCode};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "pyst")]
#[command(about = "A modern, ergonomic command runner for the Python ecosystem")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Override context
    #[arg(long, global = true)]
    context: Option<String>,
    
    /// Override config file path
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    
    /// Disable cache
    #[arg(long, global = true)]
    no_cache: bool,
    
    /// Disable color output
    #[arg(long, global = true)]
    no_color: bool,
    
    /// Control color output
    #[arg(long, global = true, value_enum)]
    color: Option<ColorMode>,
    
    /// Disallow network access
    #[arg(long, global = true)]
    offline: bool,
    
    /// Override execution directory
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,
    
    /// Pass additional flags to uv
    #[arg(long, global = true)]
    uv_flags: Option<String>,
    
    /// Verbose output
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Quiet output
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(clap::ValueEnum, Clone)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
enum Commands {
    /// List available scripts (default command)
    #[command(alias = "ls")]
    List {
        /// Show all scripts including disabled ones
        #[arg(long)]
        all: bool,
        
        /// Output format
        #[arg(long, default_value = "human")]
        format: ListFormat,
    },
    
    /// Execute a script
    Run {
        /// Script name or path
        script: String,
        
        /// Force execution bypassing context rules
        #[arg(long)]
        force: bool,
        
        /// Show what would be executed without running
        #[arg(long)]
        dry_run: bool,
        
        /// Arguments to pass to the script
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Show detailed information about a script
    Info {
        /// Script name or path
        script: String,
    },
    
    /// Print the absolute path to a script
    Which {
        /// Script name
        script: String,
    },
    
    /// Explain why a script is enabled/disabled
    Explain {
        /// Script name
        script: String,
        
        /// Output format
        #[arg(long, default_value = "human")]
        format: ExplainFormat,
    },
    
    /// Install a script from a URL or GitHub
    Install {
        /// Source URL, GitHub repo, or Gist
        source: String,
        
        /// Install with a custom name
        #[arg(long)]
        r#as: Option<String>,
    },
    
    /// Uninstall a managed script
    Uninstall {
        /// Script name
        script: String,
    },
    
    /// Update an installed script
    Update {
        /// Script name
        script: String,
    },
    
    /// Mark a script or directory as trusted
    Trust {
        /// Script name or directory path
        target: String,
    },
    
    /// Generate or manage documentation
    Document {
        /// Script name
        script: String,
        
        /// Write documentation to file
        #[arg(long)]
        write: bool,
        
        /// Check if documentation is up to date
        #[arg(long)]
        check: bool,
    },
    
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    
    /// Manage introspection cache
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    
    /// Start MCP server
    Mcp {
        /// Port for TCP transport
        #[arg(long, default_value = "8080")]
        port: u16,
        
        /// Transport type
        #[arg(long, default_value = "stdio")]
        transport: McpTransport,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum ListFormat {
    Human,
    Json,
    Markdown,
}

#[derive(clap::ValueEnum, Clone)]
enum ExplainFormat {
    Human,
    Json,
}

#[derive(Subcommand)]
enum CacheAction {
    /// Clear the cache
    Clear,
    /// Show cache path
    Path,
}

#[derive(clap::ValueEnum, Clone)]
enum McpTransport {
    Stdio,
    Tcp,
}

#[tokio::main]
async fn main() {
    let result = run().await;
    
    match result {
        Ok(exit_code) => process::exit(exit_code.into()),
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(ExitCode::GenericError.into());
        }
    }
}

async fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let context = Context::new()?;
    
    // If no subcommand is provided, default to list
    let command = cli.command.unwrap_or(Commands::List {
        all: false,
        format: ListFormat::Human,
    });
    
    match command {
        Commands::List { all, format } => {
            handle_list(&context, all, format).await
        }
        Commands::Run { script, force, dry_run, args } => {
            handle_run(&context, &script, force, dry_run, args).await
        }
        Commands::Info { script } => {
            handle_info(&context, &script).await
        }
        Commands::Which { script } => {
            handle_which(&context, &script).await
        }
        Commands::Explain { script, format } => {
            handle_explain(&context, &script, format).await
        }
        Commands::Install { source, r#as } => {
            handle_install(&context, &source, r#as.as_deref()).await
        }
        Commands::Uninstall { script } => {
            handle_uninstall(&context, &script).await
        }
        Commands::Update { script } => {
            handle_update(&context, &script).await
        }
        Commands::Trust { target } => {
            handle_trust(&context, &target).await
        }
        Commands::Document { script, write, check } => {
            handle_document(&context, &script, write, check).await
        }
        Commands::Completions { shell } => {
            handle_completions(shell).await
        }
        Commands::Cache { action } => {
            handle_cache(&context, action).await
        }
        Commands::Mcp { port, transport } => {
            handle_mcp(&context, port, transport).await
        }
    }
}

async fn handle_list(context: &Context, all: bool, format: ListFormat) -> Result<ExitCode> {
    use pyst_lib::introspection::runner::IntrospectionRunner;
    use pyst_lib::Installer;
    
    let discovery = Discovery::new(context.config.clone());
    let mut scripts = discovery.discover_scripts(context.project_root.as_deref())?;
    
    // Get installed scripts
    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("pyst")
        .join("scripts");
    let installer = Installer::new(install_dir);
    
    if let Ok(installed_scripts) = installer.list_installed() {
        // Convert installed scripts to ScriptInfo format
        for installed in installed_scripts {
            let script_info = pyst_lib::ScriptInfo {
                name: installed.name.clone(),
                path: installed.install_path.clone(),
                is_local: false, // Installed scripts are global
                description: None, // Will be filled by introspection
                entry_point: pyst_lib::EntryPoint::Unknown,
            };
            scripts.push(script_info);
        }
    }
    
    // Get introspection data for enhanced output
    let mut runner = IntrospectionRunner::new(context.config.clone())?;
    let script_paths: Vec<_> = scripts.iter().map(|s| s.path.clone()).collect();
    let introspection_results = runner.introspect_batch(&script_paths)?;
    
    // Determine active context for filtering
    let active_context = std::env::var("PYST_CONTEXT").unwrap_or_else(|_| "default".to_string());
    
    match format {
        ListFormat::Human => {
            if scripts.is_empty() {
                println!("No scripts found");
            } else {
                for (script, introspection) in scripts.iter().zip(introspection_results.iter()) {
                    let scope = if script.is_local { "local" } else { "global" };
                    let enabled = context.config.contexts.is_script_enabled(&active_context, &script.name);
                    
                    if !all && !enabled {
                        continue; // Skip disabled scripts unless --all is used
                    }
                    
                    let status = if enabled { "" } else { " (disabled)" };
                    let description = introspection.metadata.description.as_deref().unwrap_or("");
                    
                    if description.is_empty() {
                        println!("{} ({}){}", script.name, scope, status);
                    } else {
                        println!("{} ({}) - {}{}", script.name, scope, description, status);
                    }
                }
            }
        }
        ListFormat::Json => {
            use serde::Serialize;
            
            #[derive(Serialize)]
            struct EnhancedScriptInfo {
                #[serde(flatten)]
                script: pyst_lib::ScriptInfo,
                introspection: pyst_lib::introspection::schema::IntrospectionResult,
                enabled: bool,
            }
            
            let enhanced_scripts: Vec<_> = scripts.iter().zip(introspection_results.iter()).map(|(script, introspection)| {
                let enabled = context.config.contexts.is_script_enabled(&active_context, &script.name);
                EnhancedScriptInfo {
                    script: script.clone(),
                    introspection: introspection.clone(),
                    enabled,
                }
            }).filter(|s| all || s.enabled).collect();
            
            println!("{}", serde_json::to_string_pretty(&enhanced_scripts)?);
        }
        ListFormat::Markdown => {
            println!("# Available Scripts\n");
            for (script, introspection) in scripts.iter().zip(introspection_results.iter()) {
                let scope = if script.is_local { "local" } else { "global" };
                let enabled = context.config.contexts.is_script_enabled(&active_context, &script.name);
                
                if !all && !enabled {
                    continue;
                }
                
                let status = if enabled { "" } else { " *(disabled)*" };
                let description = introspection.metadata.description.as_deref().unwrap_or("");
                
                if description.is_empty() {
                    println!("- **{}** ({}){}", script.name, scope, status);
                } else {
                    println!("- **{}** ({}) - {}{}", script.name, scope, description, status);
                }
            }
        }
    }
    
    Ok(ExitCode::Success)
}

async fn handle_run(context: &Context, script: &str, force: bool, dry_run: bool, args: Vec<String>) -> Result<ExitCode> {
    let discovery = Discovery::new(context.config.clone());
    let script_info = match discovery.resolve_script(script, context.project_root.as_deref()) {
        Ok(info) => info,
        Err(_) => return Ok(ExitCode::ScriptNotFound),
    };
    
    // Check context rules unless --force is used
    if !force {
        let active_context = std::env::var("PYST_CONTEXT").unwrap_or_else(|_| "default".to_string());
        let is_enabled = context.config.contexts.is_script_enabled(&active_context, &script_info.name);
        
        if !is_enabled {
            println!("Script '{}' is disabled in context '{}'", script_info.name, active_context);
            println!("Use --force to bypass context rules, or run 'pyst explain {}' for details", script_info.name);
            return Ok(ExitCode::ExecutionBlocked);
        }
    }
    
    let executor = Executor::new(context.config.clone());
    executor.run_script(&script_info.path, &args, force, dry_run).await
}

async fn handle_info(context: &Context, script: &str) -> Result<ExitCode> {
    use pyst_lib::introspection::runner::IntrospectionRunner;
    
    let discovery = Discovery::new(context.config.clone());
    let script_info = match discovery.resolve_script(script, context.project_root.as_deref()) {
        Ok(info) => info,
        Err(_) => return Ok(ExitCode::ScriptNotFound),
    };
    
    // Get enhanced introspection data
    let mut runner = IntrospectionRunner::new(context.config.clone())?;
    let introspection = runner.introspect(&script_info.path)?;
    
    // Check context status
    let active_context = std::env::var("PYST_CONTEXT").unwrap_or_else(|_| "default".to_string());
    let enabled = context.config.contexts.is_script_enabled(&active_context, &script_info.name);
    let trusted = runner.is_trusted(&script_info.path);
    
    println!("Name: {}", script_info.name);
    println!("Path: {}", script_info.path.display());
    println!("Scope: {}", if script_info.is_local { "local" } else { "global" });
    println!("Status: {}", if enabled { "enabled" } else { "disabled" });
    println!("Trusted: {}", if trusted { "yes" } else { "no" });
    println!("Entry Point: {:?}", script_info.entry_point);
    
    if let Some(desc) = &introspection.metadata.description {
        println!("Description: {}", desc);
    }
    
    if let Some(docstring) = &introspection.metadata.docstring {
        println!("Docstring: {}", docstring);
    }
    
    if let Some(pep723) = &introspection.metadata.pep723_metadata {
        println!("PEP 723 Dependencies: {:?}", pep723.dependencies);
        if let Some(requires_python) = &pep723.requires_python {
            println!("Requires Python: {}", requires_python);
        }
    }
    
    if !introspection.metadata.dependencies.is_empty() {
        println!("Dependencies:");
        for dep in &introspection.metadata.dependencies {
            let version = dep.version_spec.as_deref().unwrap_or("");
            println!("  - {} {} (from {:?})", dep.name, version, dep.source);
        }
    }
    
    if let Some(cli_framework) = &introspection.metadata.cli_framework {
        println!("CLI Framework: {}", cli_framework.name);
        if let Some(version) = &cli_framework.version {
            println!("Framework Version: {}", version);
        }
    }
    
    if !introspection.metadata.functions.is_empty() {
        println!("Functions: {}", introspection.metadata.functions.len());
        for func in &introspection.metadata.functions {
            if func.name == "main" || func.decorators.iter().any(|d| d.contains("command")) {
                println!("  - {} (line {})", func.name, func.line_number);
                if let Some(doc) = &func.docstring {
                    println!("    {}", doc.lines().next().unwrap_or(""));
                }
            }
        }
    }
    
    if !introspection.metadata.errors.is_empty() {
        println!("Introspection Errors:");
        for error in &introspection.metadata.errors {
            println!("  - {:?}: {}", error.error_type, error.message);
        }
    }
    
    Ok(ExitCode::Success)
}

async fn handle_which(context: &Context, script: &str) -> Result<ExitCode> {
    let discovery = Discovery::new(context.config.clone());
    match discovery.resolve_script(script, context.project_root.as_deref()) {
        Ok(script_info) => {
            println!("{}", script_info.path.display());
            Ok(ExitCode::Success)
        }
        Err(_) => Ok(ExitCode::ScriptNotFound)
    }
}

async fn handle_explain(context: &Context, script: &str, format: ExplainFormat) -> Result<ExitCode> {
    // Determine active context
    let active_context = std::env::var("PYST_CONTEXT").unwrap_or_else(|_| "default".to_string());
    
    // Evaluate the script in the active context
    let evaluation = context.config.contexts.evaluate_script(&active_context, script);
    
    match format {
        ExplainFormat::Human => {
            println!("Script: {}", evaluation.script_name);
            println!("Context: {}", evaluation.context_name);
            println!("Status: {}", if evaluation.enabled { "ENABLED" } else { "DISABLED" });
            println!();
            
            if let Some(final_rule) = &evaluation.final_rule {
                println!("Final determining rule:");
                println!("  Pattern: {}", final_rule.pattern);
                println!("  Type: {}", if final_rule.is_negation { "Exclusion (!)" } else { "Inclusion" });
                println!("  Matches: {}", final_rule.matches);
                println!();
            }
            
            println!("All rules in context:");
            for (index, rule) in evaluation.all_rules.iter().enumerate() {
                let status = if rule.matches {
                    if rule.is_negation { "MATCHES (disables)" } else { "MATCHES (enables)" }
                } else {
                    "no match"
                };
                println!("  {}: {} -> {}", index + 1, rule.pattern, status);
            }
            
            if evaluation.matched_rules.len() > 1 {
                println!();
                println!("Note: Multiple rules matched. The last matching rule takes precedence.");
            }
        }
        ExplainFormat::Json => {
            let json_output = serde_json::to_string_pretty(&evaluation)?;
            println!("{}", json_output);
        }
    }
    
    Ok(ExitCode::Success)
}

async fn handle_install(context: &Context, source: &str, name: Option<&str>) -> Result<ExitCode> {
    use pyst_lib::Installer;
    
    // Get global script directory from config
    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("pyst")
        .join("scripts");
    
    let installer = Installer::new(install_dir);
    
    println!("Installing from: {}", source);
    if let Some(name) = name {
        println!("Custom name: {}", name);
    }
    
    match installer.install(source, name).await {
        Ok(installed_scripts) => {
            if installed_scripts.is_empty() {
                println!("No Python scripts found to install");
            } else {
                println!("Successfully installed {} script(s):", installed_scripts.len());
                for script in &installed_scripts {
                    println!("  - {}", script);
                }
                
                if installed_scripts.len() == 1 {
                    println!("\nYou can now run: pyst run {}", installed_scripts[0]);
                } else {
                    println!("\nYou can now run any of these scripts with: pyst run <script-name>");
                }
                
                println!("\nTo see all installed scripts: pyst list --all");
            }
            Ok(ExitCode::Success)
        }
        Err(err) => {
            eprintln!("Failed to install: {}", err);
            Ok(ExitCode::GenericError)
        }
    }
}

async fn handle_uninstall(context: &Context, script: &str) -> Result<ExitCode> {
    use pyst_lib::Installer;
    
    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("pyst")
        .join("scripts");
    
    let installer = Installer::new(install_dir);
    
    match installer.uninstall(script).await {
        Ok(()) => {
            println!("Successfully uninstalled script: {}", script);
            Ok(ExitCode::Success)
        }
        Err(err) => {
            eprintln!("Failed to uninstall: {}", err);
            Ok(ExitCode::GenericError)
        }
    }
}

async fn handle_update(context: &Context, script: &str) -> Result<ExitCode> {
    use pyst_lib::Installer;
    
    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("pyst")
        .join("scripts");
    
    let installer = Installer::new(install_dir);
    
    println!("Updating script: {}", script);
    
    match installer.update(script).await {
        Ok(()) => {
            println!("Successfully updated script: {}", script);
            Ok(ExitCode::Success)
        }
        Err(err) => {
            eprintln!("Failed to update: {}", err);
            Ok(ExitCode::GenericError)
        }
    }
}

async fn handle_trust(context: &Context, target: &str) -> Result<ExitCode> {
    use pyst_lib::introspection::runner::IntrospectionRunner;
    
    let mut runner = IntrospectionRunner::new(context.config.clone())?;
    let target_path = std::path::PathBuf::from(target);
    
    if !target_path.exists() {
        println!("Error: Path does not exist: {}", target);
        return Ok(ExitCode::GenericError);
    }
    
    runner.trust_path(&target_path)?;
    
    if target_path.is_dir() {
        println!("Trusted directory: {}", target_path.display());
        println!("All scripts in this directory can now use import-mode introspection");
    } else {
        println!("Trusted script: {}", target_path.display());
        println!("This script can now use import-mode introspection");
    }
    
    Ok(ExitCode::Success)
}

async fn handle_document(context: &Context, _script: &str, _write: bool, _check: bool) -> Result<ExitCode> {
    println!("Document command - not yet implemented");
    Ok(ExitCode::Success)
}

async fn handle_completions(_shell: clap_complete::Shell) -> Result<ExitCode> {
    println!("Completions command - not yet implemented");
    Ok(ExitCode::Success)
}

async fn handle_cache(context: &Context, action: CacheAction) -> Result<ExitCode> {
    use pyst_lib::introspection::runner::IntrospectionRunner;
    
    let mut runner = IntrospectionRunner::new(context.config.clone())?;
    
    match action {
        CacheAction::Clear => {
            runner.clear_cache()?;
            println!("Cache cleared successfully");
        }
        CacheAction::Path => {
            println!("{}", runner.get_cache_path().display());
        }
    }
    
    Ok(ExitCode::Success)
}

async fn handle_mcp(context: &Context, _port: u16, _transport: McpTransport) -> Result<ExitCode> {
    println!("MCP command - not yet implemented");
    Ok(ExitCode::Success)
}