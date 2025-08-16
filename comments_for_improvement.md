Great report and thanks for pasting the tree and sources. You’ve already got a lot of the scaffolding in place; the “run does nothing” symptom is a concrete clue. Here’s a focused plan across four buckets.

1) Fixes needed in the current codebase

High-priority runtime fixes
- Stream child process output in `pyst run`
  - Current behavior captures stdout/stderr and discards them (never printed), so commands appear to “do nothing.”
  - In `pyst-lib/src/executor/runner.rs`, switch from `output()` to either:
    - fully inheriting stdio and using `status()`:
      - `cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());`
      - `let status = cmd.status()?;`
    - or capture and print buffers explicitly.
  - Also add the `--` separator so `uv` doesn’t confuse script args as its own flags:
    - `uv run <script.py> -- <args...>`

- Respect working directory
  - Set `Command::current_dir(...)` based on `core.cwd` and CLI `--cwd`.
  - Use project root when `cwd = "project"` (as you specified), or fallback to script dir/current dir as needed.

- Apply offline and uv flags
  - Offline: set `UV_NO_NETWORK=1` when `--offline` or `core.offline = true`.
  - uv flags precedence: CLI `--uv-flags` > `PYST_UV_FLAGS` > `[core.uv].flags`.
  - Append to `cmd.args(...)` before `--`.

- Always pass `--` between the script path and user args
  - In `Executor::run_script`, after `cmd.arg(script_path)`, add `cmd.arg("--")`, then append `args`.

- Honor CLI/global flags
  - CLI `--context`: you’re reading `PYST_CONTEXT` elsewhere, but never set it. At the start of `run()` (after parsing CLI), set `std::env::set_var("PYST_CONTEXT", <cli value>)` if provided.
  - CLI `--offline`, `--uv-flags`, `--cwd`, `--no-cache` should override the loaded config for that process run. Today, `Context::new()` loads config before you parse CLI. Invert the order or add a `Context::new_with_overrides(cli: &Cli)` that:
    - calls `Config::load_with_override(cli.config)`,
    - then applies per-run overrides to the loaded `Config` instance (offline, uv flags, cwd),
    - and exports `PYST_CONTEXT` if provided.
  - CLI `--no-cache`: plumb to `IntrospectionRunner` (see below).

- MCP: don’t shell out to `./target/debug/pyst`
  - In `pyst-lib/src/mcp.rs` `tool_run_script`, use `std::env::current_exe()` (or just re-enter the library logic directly) instead of a debug-only path that will break in release installs.

- Fix `~` expansion
  - In `config::Config::expand_path`, `&path_str[2..]` assumes `~/`. Use a robust expander:
    - If `path_str == "~"`, use `home`.
    - If `path_str.starts_with("~/")`, slice `[2..]`.
    - Else leave unchanged.

- Handle `--dry-run` copy text
  - Your test expects “Would execute …” and code already prints that; keep consistent and include the full uv command with `--`.

- Separate scripts added by installer
  - You add installed scripts to the list. Consider avoiding duplicates if a local script has same name.

Introspection/caching toggles
- `--no-cache` integration
  - Add a boolean `no_cache` to `IntrospectionRunner` that:
    - bypasses `cache.get` reads,
    - and/or skips `cache.put`.
  - Wire CLI global `--no-cache` to this.

- Offline for introspector
  - If offline, set `UV_NO_NETWORK=1` when running `uv run introspector.py` (even though introspector has no third-party deps, uv may still want the managed interpreter).

- Click/typer entry detection (minor bug)
  - In `introspector.py`, the click decorator test uses `decorator.id` for `ast.Attribute` nodes; that attribute won’t exist. Consider a simpler heuristic or just rely on import detection for now.

2) Tests to add or make more thorough

Run path and output
- Streaming stdout/stderr
  - Create a `.pyst/hello.py` printing to stdout/stderr; assert runtime output appears. Test both `print` and `click/typer` scripts.
- Argument forwarding
  - Test a script consuming positional args and flags (including edge cases like `--`, `-n`, quotes, spaces). Ensure they reach the script unchanged.
- Exit codes
  - A script that calls `sys.exit(5)` should cause `pyst run` to return nonzero. Decide whether to bubble the code or map to generic; test per your policy.
- Working directory
  - Script that writes `cwd.txt` showing `os.getcwd()`; assert it matches project root by default. Test `--cwd` override and config `cwd = "script"`.

Config/CLI precedence
- `--context` overrides
  - Ensure a script disabled by default context is runnable with `--force`, and ensure `--context` takes precedence over `PYST_CONTEXT`.
- `--offline`
  - A PEP 723 script with a new dependency: `pyst run` should fail with exit code 102 in offline mode if env is not already hydrated.
- `--uv-flags`
  - Pass a benign flag (e.g., `--python-preference=managed`) and verify it reaches child process (if you can introspect via `ps` in CI; otherwise, mock).

Introspection and cache
- `--no-cache` actually bypasses cache
  - Modify a script description and verify `list`/`info` reflects it even if cache would say otherwise.
- Cache invalidation on file change and Python version change.

Name resolution and selectors
- `project:` vs `global:` explicit selection when collisions exist.
- Path invocation `pyst run ./relative/path.py` respects contexts (blocked unless `--force`, as per your spec).

Installer (since it’s present)
- Smoke tests for `install` from a gist/raw URL, then `list` sees it and `run` works.
- `uninstall` removes file and updates manifest.

MCP server (basic)
- Start stdio server and send `initialize`, `tools/list`, `tools/call` with `list_scripts`. Assert non-error replies.

Cross-platform
- Windows argument quoting and `--` handling (if you can add a Windows runner).
- Paths with spaces.

3) What’s left to implement from the plan

- Wire CLI overrides into config/runtime
  - `--context`, `--config`, `--offline`, `--uv-flags`, `--cwd`, `--no-cache`, `--no-color`, `--color`, `-v/-q`. Right now most are parsed but unused.
- Executor polish
  - Precedence of uv flags, `--` separator, env `UV_NO_NETWORK`, `current_dir`, and stream stdout/stderr.
- Completions
  - Implement with `clap_complete`. Persist to stdout; users redirect as needed.
- Introspection
  - Batch mode (single uv process for many files) is planned but not implemented.
  - Import-mode enhancement (trusted paths) is scaffolded but not adding extra info yet; OK to defer.
- Contexts
  - Inheritance via `extends` (out of scope for your current code).
  - `pyst explain --format json` is done; good. Consider printing config provenance of the rule later.
- Install/update/shims
  - You have install/uninstall/update and a manifest; still missing PATH shims in `~/.local/share/pyst/bin` (optional if you want direct `name` execution).
- MCP
  - TCP transport (you already fallback to stdio).
  - Tools: You’ve got the core four. Consider returning structured JSON, not human text, for MCP `result.content` if the client expects it.
- Document command
  - Present but tied to `dspy` and OpenRouter config; no provider abstraction yet. It’s fine to ship later.

4) UX simplifications and consistency

- Make `pyst run` feel immediate
  - Stream stdout/stderr by default; that alone fixes the “nothing happens” problem.
  - Show the actual command in `--dry-run`, including cwd, env deltas, and uv flags.

- Context clarity
  - On `run` block, print a one-liner with the rule that blocked it, e.g., “blocked by rule ‘!db-*’ in context ‘default’ (last match wins). Use ‘pyst explain <name>’ for details.”
  - You’ve got `explain`; make sure `run` points to it.

- Uniform application of contexts
  - You elected to apply contexts even for absolute paths (unless `--force`). Keep that consistent and loud in docs.

- `--format markdown`
  - For `list`: a single table line per script is easier to copy into docs.
  - For `info`: include a compact parameters section later when structured CLI param introspection lands.

- Error messages
  - If `uv` is missing, show a helpful one-liner with install instructions. If `--offline` prevents managed Python download, exit 102 with a clear hint.

- Defaults surfaced
  - `pyst --version --diagnostics` (future) printing config dirs, cache dir, active context, uv flags, cwd policy can save user time.

- MCP result shape
  - For tools in MCP, prefer returning structured JSON and let clients render; today you return a single text blob. Consider adding a `content: [{"type":"json","json":{...}}]` mode.

Notes for document provider config
- You’re defaulting to OpenRouter for LLMs in README, which is fine. If you plan to support Anthropic/OpenAI natively later, link to their docs so users can map credentials:
  - Anthropic API docs: [docs.anthropic.com](https://docs.anthropic.com/en/home)
  - OpenRouter model endpoints (e.g., Claude 3/3.5 Haiku variants): [openrouter.ai](https://openrouter.ai/anthropic/claude-3-haiku), [openrouter.ai](https://openrouter.ai/anthropic/claude-3.5-haiku/api)
- If you host raw test scripts on Cloudflare R2 or Workers, check their developer platform docs and file handling limits: [developers.cloudflare.com](https://developers.cloudflare.com/developer-platform/llms-full.txt)

Concrete code changes (sketch)

In `pyst-lib/src/executor/runner.rs`:
- Stream and add `--`, cwd, env

  use std::process::{Command, Stdio};

  pub async fn run_script(&self, script_path: &PathBuf, args: &[String], _force: bool, dry_run: bool) -> Result<ExitCode> {
      let mut cmd = Command::new("uv");
      cmd.arg("run")
         .arg(script_path);

      // uv flags precedence: CLI > env > config
      for flag in self.resolve_uv_flags() {
          cmd.arg(flag);
      }

      // Separator between uv args and script args
      cmd.arg("--");
      for arg in args {
          cmd.arg(arg);
      }

      // CWD policy
      if let Some(dir) = self.resolve_cwd_for(script_path)? {
          cmd.current_dir(dir);
      }

      // Offline
      if self.config.core.offline {
          cmd.env("UV_NO_NETWORK", "1");
      }

      if dry_run {
          println!("Would execute: {:?}", cmd);
          return Ok(ExitCode::Success);
      }

      // Stream stdio
      let status = cmd.stdin(Stdio::inherit())
                      .stdout(Stdio::inherit())
                      .stderr(Stdio::inherit())
                      .status()?;

      if status.success() {
          Ok(ExitCode::Success)
      } else {
          // Map or propagate
          let code = status.code().unwrap_or(1);
          if code == 127 { Ok(ExitCode::ScriptNotFound) } else { Ok(ExitCode::GenericError) }
      }
  }

- Add helpers `resolve_uv_flags()` and `resolve_cwd_for(...)` that use config + CLI overrides (you can store run-time overrides in `Context` or pass them into `Executor::new`).

In `pyst/src/main.rs`:
- Apply CLI overrides before creating `Context` or pass them into `Context::new_with_overrides(&cli)`.
- Set `PYST_CONTEXT` if `cli.context` is provided.
- Thread `cli.no_cache`, `cli.offline`, `cli.cwd`, `cli.uv_flags` through to `Executor` and `IntrospectionRunner`.

In `pyst-lib/src/mcp.rs`:
- Replace `Command::new("./target/debug/pyst")` with `Command::new(std::env::current_exe()?)` or, better, call into the library directly to avoid a subprocess hop.

Summary of priority order
- P0: Executor streaming/stdout + `--` + cwd + offline/uv flags + CLI overrides applied; MCP path fix.
- P1: `--no-cache` plumbed; better error messages; tests for run/args/cwd/contexts/offline; completions.
- P2: Batch introspection; more robust entry detection; PATH shims; MCP structured results; provider abstraction for `document`.

Once you apply the P0 fixes, your “pyst run hello” will behave as expected and most other workflows will feel immediately solid.