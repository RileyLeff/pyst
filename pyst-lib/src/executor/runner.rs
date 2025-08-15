use crate::{Config, ExitCode};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub struct Executor {
    config: Config,
}

impl Executor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run_script(
        &self,
        script_path: &PathBuf,
        args: &[String],
        _force: bool,
        dry_run: bool,
    ) -> Result<ExitCode> {
        if dry_run {
            println!("Would execute: uv run {} {}", script_path.display(), args.join(" "));
            return Ok(ExitCode::Success);
        }

        let mut cmd = Command::new("uv");
        cmd.arg("run").arg(script_path);
        
        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd.output()?;
        
        if output.status.success() {
            Ok(ExitCode::Success)
        } else {
            let exit_code = output.status.code().unwrap_or(1);
            if exit_code == 127 {
                Ok(ExitCode::ScriptNotFound)
            } else {
                Ok(ExitCode::GenericError)
            }
        }
    }
}