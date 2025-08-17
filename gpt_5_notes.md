Product features still to implement
- Introspection
  - Batch introspection: `IntrospectionRunner::introspect_batch` currently shells out per file. Add a single “batch mode” `uv run introspector.py --batch <json-list>` to amortize uv startup. You can keep the per-file path as fallback.
  - Import-mode enhancement: `ImportIntrospector._enhance_with_imports()` is a stub. For trusted scripts, import the module in a sandbox (set `PYST_INTROSPECT=1`, `NO_COLOR=1`, maybe `UV_NO_NETWORK=1` when offline) and extract:
    - Resolved CLI entry object for Typer/Click,
    - Parameter names/defaults for `main`, and
    - Version data for frameworks if importable.
- Contexts
  - Inheritance: `extends` and merge logic for `[contexts]`.
  - Provenance in explain: include file path and index of the final rule (e.g., `~/.config/pyst/pyst.toml:contexts.default.enabled[2]`).
- Install/update/shims
  - PATH shims: create shims in `~/.local/share/pyst/bin` (plus Windows `.cmd`/`.ps1`) that call `pyst run <name> -- "$@"`. Optional: a “direct uv” shim for power users.
  - Integrity and pinning: you already store `sha256` and `commit_sha` for GitHub; verify hashes on run when a `--integrity` flag is set.
- MCP
  - Structured JSON results: today you return human text in `result.content`. Add a JSON content alternative: `{"type":"json","json":{...}}` alongside text to help clients consume data reliably.
  - TCP transport and basic auth later. For background/examples of MCP dev flow and tools, see blogs outlining Python SDKs and dev tools [streetsdigital.com](https://streetsdigital.com/2025/04/04/mcp-python-sdk/).
- Document command
  - Provider abstraction (OpenAI/Anthropic/Gemini/local) around your DSPy helper so the model/backend is configurable without code changes.

Correctness and performance gaps
- Deduplication in list
  - `pyst list` appends installed scripts after discovered ones and can show duplicates by name. Collapse to a single “effective” entry by precedence (local vs global). Keep `--all` to show both origins.
- Cache stats and portability
  - `Cache::get_stats` can’t validate entries because `key_to_path` always errors. Fix by storing the absolute `path` in `CacheEntry` (add a `path: String`) or by using the path itself as the cache key. Then stats can be accurate.
  - You improved environment parity by reading Python version via `uv run python --version`; that better matches the interpreter uv will use [pypi.org](https://pypi.org/project/uv/).
- Click decorator detection in `introspector.py`
  - The `decorator.id` check for `ast.Attribute` is incorrect. Either:
    - rely on import-level detection (`import click`/`from click`), or
    - handle `ast.Call` and `ast.Attribute` properly when decorator is `click.command`.
- Helper script path robustness
  - `get_introspector_path`/`find_documenter_helper` rely on relative locations next to the binary. Consider:
    - embedding helpers with `include_str!` and writing to a temp file, or
    - copying helpers to `~/.local/share/pyst/helpers/` on first run and using that path thereafter.
- Windows flags not wired
  - `--color/--no-color`, `-v/-q` are parsed but unused. Either wire up a minimal logging/color layer (`tracing` + a simple color lib) or remove the flags from CLI until ready, to avoid user confusion.

UX polish
- Blocked run message
  - When a run is blocked, also print the final rule inline (pattern + context), not just “use explain.”
- `pyst list --format markdown`
  - Output a simple table (Name | Origin | Enabled | Summary) for easy copy/paste into docs.
- Diagnostics
  - Add `pyst doctor` (or `--diagnostics`) to print: active context, cwd policy, uv flags, resolved config paths, cache dir, uv availability/version.

Tests to add (or extend)
- Dedup and precedence
  - Create local and installed `hello`; ensure `list` shows one effective entry; `--all` shows both; and `project:hello`/`global:hello` resolve explicitly.
- Offline 102 and prewarm
  - Use `.pyst/test-offline.py` which requires `requests`. Assert `pyst run test-offline --offline` exits 102 when environment isn’t prewarmed; succeeds when online once.
- `--color`, `-v/-q` (if you keep them)
  - Either skip or implement minimal behavior and add a smoke test verifying log level/ANSI toggles.
- Windows quoting/paths with spaces
  - Add native Windows test (simple) for `pyst run` with args containing spaces/special chars and file under a path with spaces.
- MCP stdio smoke
  - Start server, send `initialize` and `tools/list`, assert the response is non-error. Optionally, call `run_script` for a trivial script and check text content.

CI issues to fix (from ci_problems.md)
- Windows PowerShell “if” syntax error
  - Your CI is executing a Bash conditional in PowerShell. For steps that need Bash, set `shell: bash` explicitly:
    - Example:
      ```yaml
      - name: Some bash step
        shell: bash
        run: |
          if [ "$RUN_ARM64" = "1" ]; then
            echo "..."
          fi
      ```
- Linux ARM64 apt packages not found
  - On Ubuntu 24.04 (Noble), `pkg-config-aarch64-linux-gnu` and `libssl-dev:arm64` may be unavailable/misaligned. Options:
    - Easiest: drop the native ARM64 job for now (keep x86_64 native + Linux container tests).
    - Or use cross-compilation with vendored OpenSSL only (no apt): `OPENSSL_STATIC=1 OPENSSL_VENDORED=1` and avoid architecture-specific apt packages.
- Container image not found in some jobs
  - The failing tests tried to pull `pyst-test:latest` from a job that hadn’t built it. Ensure only the “container-tests” job runs tests that depend on that image (you are already doing this). If other jobs still run those tests:
    - Mark container-based tests with a cargo feature and only enable that feature in the container-tests job, or
    - Add a runtime guard in tests: if `docker image inspect pyst-test:latest` fails, skip those tests.
- Codecov 429 rate limit
  - For public repos, tokenless uploads usually work, but rate limits happen. Solutions:
    - Add `CODECOV_TOKEN` as a repo secret and configure the action to use it (recommended).
    - Or set `fail_ci_if_error: false` (you already do) and keep `continue-on-error: true` to avoid red builds when rate-limited.

Nice-to-haves aligned with ecosystem
- uv integration and docs
  - Your design aligns with uv’s core capabilities: runs scripts with inline dependency metadata per PEP 723, manages Python versions, and installs tools [pypi.org](https://pypi.org/project/uv/). For teams considering uv vs other tools, there’s good independent context on migration and speed benefits [thijsnieuwdorp.com](https://thijsnieuwdorp.com/posts/uv/) and notes on Python version handling sourced from `python-build-standalone` [x-cmd.com](https://www.x-cmd.com/pkg/uv/).
- MCP notes
  - As you add structured JSON responses and TCP later, those workflows align with how MCP SDKs and dev tools are used in practice [streetsdigital.com](https://streetsdigital.com/2025/04/04/mcp-python-sdk/).

Quick code pointers (where to change)
- Batch introspection: `pyst-lib/helpers/introspector.py` (add `--batch`); `pyst-lib/src/introspection/runner.rs` (serialize N paths, call once).
- Dedup: `pyst/src/main.rs` in `handle_list` before formatting output; keep a `HashMap<String, (origin_rank, ScriptInfo, IntrospectionResult)>` keyed by name and apply precedence.
- Cache stats: `pyst-lib/src/introspection/cache.rs` add `path` in `CacheEntry`, populate in `put`, use it in `get_stats`.
- Click decorator detection: `pyst-lib/helpers/introspector.py::_extract_entry_points`.
- Shims: new module under `pyst-lib/src/install/shims.rs`; created during install; add `pyst install` post-step to write shims and print PATH tip.

Minimal roadmap (ready-to-ship order)
1) Dedup in list; blocked-run message shows final rule; fix Click decorator detection; cache stats fix. Add offline 102 test.
2) Batch introspection; structured MCP JSON outputs (keep text too).
3) PATH shims; context `extends` + provenance in `explain`.
4) Document provider abstraction; MCP TCP transport.

With these wrapped, you’ll have a very complete v0.1 that feels instantaneous and predictable, while staying aligned with uv’s strengths (inline metadata, fast envs, Python version management) [pypi.org](https://pypi.org/project/uv/).

You’re seeing two separate issues:

1) “Default scripts” showing up
- pyst does not ship any scripts. Those entries are coming from your user data dir where you previously installed examples during dev.
- On macOS, `dirs::data_local_dir()` resolves to `~/Library/Application Support`, so your global scripts live at:
  - `~/Library/Application Support/pyst/scripts`
  - plus a manifest at `~/Library/Application Support/pyst/scripts/manifest.json`
- Because `pyst list` reads the manifest (not the raw directory), anything you installed earlier is still being listed.

How to reset to “no default scripts”
- Preferred: uninstall via pyst so the manifest stays consistent:
  - `pyst uninstall hello`
  - `pyst uninstall db-backup`
  - `pyst uninstall weather`
  - `pyst uninstall uuid-generator`
- Or nuke the whole global scripts dir (removes files and manifest):
  - `rm -rf ~/Library/Application\ Support/pyst/scripts`
- After that, `pyst` will list only project scripts (and nothing global unless you install again).

2) “Cannot find introspector.py helper script”
- Cause: the installed binary from `cargo install` no longer lives in your workspace, so the relative lookups in `IntrospectionRunner::get_introspector_path()` can’t find `helpers/introspector.py`.
- Fix: embed the helper at build time and write it to the user data dir (e.g., `~/.local/share/pyst/helpers` on Linux; `~/Library/Application Support/pyst/helpers` on macOS) on first use. Then always run that copy.

Minimal patch sketch

In `pyst-lib/src/introspection/runner.rs`, embed the helper and ensure it exists on disk:

- At top of file (or a small `helpers.rs` module):
  - Embed the script source:
    const INTROSPECTOR_SRC: &str = include_str!("../../helpers/introspector.py");
    // path is relative to this file: src/introspection/runner.rs -> ../../helpers/introspector.py

  - Add a helper to write it once to the data dir:
    use std::io::Write;

    fn ensure_helper_installed(config: &Config) -> anyhow::Result<std::path::PathBuf> {
        let helpers_dir = config.get_data_dir()?.join("helpers");
        std::fs::create_dir_all(&helpers_dir)?;
        let target = helpers_dir.join("introspector.py");

        // Write only if missing or stale (optional: compare a hash)
        if !target.exists() {
            let mut f = std::fs::File::create(&target)?;
            f.write_all(INTROSPECTOR_SRC.as_bytes())?;
        }
        Ok(target)
    }

- Change `get_introspector_path()` to prefer the embedded copy:
    fn get_introspector_path(&self) -> Result<PathBuf> {
        // 1) Ensure embedded helper is available in the data dir
        if let Ok(p) = ensure_helper_installed(&self.config) {
            return Ok(p);
        }

        // 2) Fallbacks for dev builds (when running from workspace)
        let current_exe = std::env::current_exe()?;
        let exe_dir = current_exe.parent().ok_or_else(|| anyhow!("Cannot find executable directory"))?;
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

Do the same for `documenter.py`
- In `pyst-lib/src/document/mod.rs`, add:
  - const DOCUMENTER_SRC: &str = include_str!("../../helpers/documenter.py");
  - a `ensure_documenter_installed(&Config) -> PathBuf` similar to above,
  - use it in `find_documenter_helper()` to return the data-dir copy first, then fall back to workspace paths.

Why this approach
- It works for `cargo install` binaries, Homebrew packages, GitHub Releases, etc. There are no fragile “relative to the executable” assumptions.
- The helper is always available for uv to run, regardless of where the binary lives.
- If you want to keep helpers updatable without a recompile, add a tiny version/hash check and overwrite the file when the embedded version changes.

Small UX touches worth adding
- When you first write helpers, print nothing (silent). Only log if the write fails.
- Add a `pyst doctor` later that prints:
  - helper locations,
  - data/cache dirs,
  - uv availability/version,
  - active context/cwd policy,
  - and confirms manifest and installed scripts.
- Consider an opt-in env `PYST_HELPERS_DIR` to override where helpers are written (useful for debugging).

Notes and references
- Your choice to rely on uv to manage Python and environments is aligned with the tooling direction in packaging—uv can even manage interpreters, making isolation fast and predictable [lucumr.pocoo.org](https://lucumr.pocoo.org/2024/8/21/harvest-season/). Treat uv as the “python -m …” of this era; it ensures the right environment runs the tool, similar in spirit to advice like “run `python -m pytest` to pick the correct interpreter” [simonwillison.net](https://simonwillison.net/2024/Aug/17/python-m-pytest/).

Summary action list
- Clean existing global installs (uninstall or delete `~/Library/Application Support/pyst/scripts`).
- Implement helper embedding + first-run extract for `introspector.py` and `documenter.py`.
- Optionally add a one-liner in README: “If you see global scripts listed, they are from previous installs in your data dir; use `pyst uninstall` or remove the manifest to reset.”

Once you embed and extract helpers, the warnings will disappear for `cargo install` builds, and an out-of-the-box install won’t show any scripts unless the user has actually installed some.

You’re hitting two separate problems:

A) run does “nothing” for installed scripts  
B) the installed binary can’t find introspector.py

Root causes and fixes

1) Name resolution excludes installed scripts
- What’s happening: `pyst list` shows installed scripts because it merges discovery with the Installer manifest. But `pyst run <name>` uses only `Discovery.resolve_script(...)`, which checks the project and configured global directories and does not consult the Installer manifest. On macOS you installed scripts under `~/Library/Application Support/pyst/scripts` (from `dirs::data_local_dir()`), but your default `[core.global_script_dirs]` points to `~/.local/share/pyst/scripts`. So run can’t find “weather”, “uuid-generator”, “db-backup” and exits 127 silently.
- Fix options (pick one):
  - Best: teach `Discovery` to also source from the Installer manifest and from the XDG data dir consistently.
    - Add a helper in `Discovery` to load installed entries via `Installer::list_installed()` and treat them as global scripts for both listing and resolution.
    - In `resolve_global_script`, check the manifest entries first (or based on precedence), then walk configured directories.
  - Also add the data-dir scripts path to discovery automatically:
    - In both `discover_scripts` and `resolve_global_script`, also scan `config.get_data_dir()?.join("scripts")`. This aligns discovery with where the installer actually writes on macOS and Windows.
  - Or change the default config to use the data dir:
    - Make the default global dir `"{DATA_DIR}/pyst/scripts"` instead of a hard-coded `~/.local/share/pyst/scripts`. On macOS that becomes `~/Library/Application Support/pyst/scripts`; on Linux it’s `~/.local/share/pyst/scripts`; on Windows it’s `%AppData%\pyst\scripts`. Using the platform’s data dir avoids surprises.

Code sketch (Discovery + manifest)
- In `pyst-lib/src/discovery.rs`, add a method to pull installed entries:

  // New
  fn list_installed_as_scripts(&self) -> Vec<ScriptInfo> {
      let mut out = Vec::new();
      if let Ok(dir) = self.config.get_data_dir() {
          let install_dir = dir.join("scripts");
          let installer = crate::install::Installer::new(install_dir);
          if let Ok(installed) = installer.list_installed() {
              for it in installed {
                  out.push(ScriptInfo {
                      name: it.name.clone(),
                      path: it.install_path.clone(),
                      is_local: false,
                      description: None,
                      entry_point: crate::discovery::EntryPoint::Unknown,
                  });
              }
          }
      }
      out
  }

- In `discover_scripts`, append those installed entries (and then deduplicate by name using precedence).
- In `resolve_script`, if not found by normal rules, check the installed scripts by manifest:

  // After scanning local/global dirs:
  let installed = self.list_installed_as_scripts();
  if let Some(s) = installed.into_iter().find(|s| s.name == name) {
      return Ok(s);
  }

- Also consider always scanning `self.config.get_data_dir()?.join("scripts")` as an additional global dir so discovery and run remain consistent even if we don’t consult the manifest.

Also add a friendlier message on run-not-found
- Today `handle_run` returns 127 without a message. Consider:

  let script_info = match discovery.resolve_script(script, ...) {
      Ok(info) => info,
      Err(_) => {
          eprintln!("Script '{}' not found. Try `pyst list` or specify `project:`/`global:` explicitly.", script);
          return Ok(ExitCode::ScriptNotFound);
      }
  };

2) The installed binary can’t find introspector.py
- What’s happening: after `cargo install`, the binary lives in `~/.cargo/bin/pyst`, but your `IntrospectionRunner::get_introspector_path()` looks for helpers relative to the executable or the workspace layout. Those paths don’t exist in an installed binary context, so introspection warns “Cannot find introspector.py helper script.”
- Fix: embed helpers at build time and extract them to the user data dir on first use. Then always run the extracted copy.

Concrete patch (minimal)

- In `pyst-lib/src/introspection/runner.rs`:

  // At top
  const INTROSPECTOR_SRC: &str = include_str!("../../helpers/introspector.py");

  fn ensure_introspector_installed(config: &Config) -> anyhow::Result<std::path::PathBuf> {
      let dir = config.get_data_dir()?.join("helpers");
      std::fs::create_dir_all(&dir)?;
      let target = dir.join("introspector.py");
      // Overwrite if missing; you can add a version/hash check to update
      if !target.exists() {
          std::fs::write(&target, INTROSPECTOR_SRC.as_bytes())?;
      }
      Ok(target)
  }

  fn get_introspector_path(&self) -> Result<PathBuf> {
      // 1) Use embedded helper at the platform data dir
      if let Ok(p) = ensure_introspector_installed(&self.config) {
          return Ok(p);
      }
      // 2) Dev fallbacks (workspace-relative) remain as last resort
      let current_exe = std::env::current_exe()?;
      let exe_dir = current_exe.parent().ok_or_else(|| anyhow!("Cannot find executable directory"))?;
      let possible_paths = [
          exe_dir.join("../pyst-lib/helpers/introspector.py"),
          exe_dir.join("helpers/introspector.py"),
          PathBuf::from("pyst-lib/helpers/introspector.py"),
      ];
      for p in &possible_paths {
          if p.exists() {
              return Ok(p.clone());
          }
      }
      Err(anyhow!("Cannot find introspector.py helper script"))
  }

- Do the same for the documenter in `pyst-lib/src/document/mod.rs`:

  const DOCUMENTER_SRC: &str = include_str!("../../helpers/documenter.py");
  fn ensure_documenter_installed(config: &Config) -> anyhow::Result<PathBuf> { ... }
  // In find_documenter_helper(), return ensure_documenter_installed(config)? first

This makes `cargo install` builds self-contained and fixes the warnings.

“Default scripts” after install
- pyst does not ship scripts. The items you’re seeing are ones you installed earlier and are stored in your user data dir:
  - macOS: `~/Library/Application Support/pyst/scripts`
- To reset:
  - Preferred: `pyst uninstall hello`, `pyst uninstall weather` etc., so the manifest stays in sync.
  - Or remove the directory: `rm -rf ~/Library/Application\ Support/pyst/scripts`.

Why using uv run is the right model
- Running tools via the environment that owns them avoids “wrong interpreter” surprises—same philosophy as “use `python -m pytest` instead of `pytest` to ensure you run the tool in the correct environment” [simonwillison.net](https://simonwillison.net/2024/Aug/17/). pyst’s `uv run` orchestration ensures the correct Python and dependencies are used.

Quick validation after fixes
- Rebuild and install: `cargo install --path .`
- Verify helpers are provisioned:
  - macOS: `ls ~/Library/Application\ Support/pyst/helpers/` should show `introspector.py` (and `documenter.py` if you open that path).
- Verify run resolution:
  - `pyst list` (no more warnings)
  - `pyst run weather` — should now execute (or block by context for db-backup with a clear message).
  - `pyst run project:hello` still works as before. If a name collision exists, `project:` and `global:` selectors resolve explicitly.

Optional quality-of-life
- Deduplicate names in `list` (show the “effective” one by precedence and display both with `--all`).
- Add a diagnostics command (`pyst doctor`) to print the resolved paths (helpers, data dir, cache dir, uv version).
- Consider updating the default global dir to the platform’s data dir to reduce surprises on macOS/Windows.

Once you embed the helpers and unify name resolution, both problems you saw (warnings and “no-op” run) will disappear.