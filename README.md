# pyst

A modern, ergonomic command runner for Python scripts. Turn local and global `.py` files into a fast, discoverable toolset. Powered by uv for isolation and speed.

- Discover scripts in your project’s `.pyst/` directory and in global script dirs
- Run with automatic environment resolution via uv
- Get rich info and JSON output for human and machine use
- Apply context rules to enable/disable scripts; bypass with --force
- Install scripts from GitHub/Gists/URLs with a managed manifest

Status at a glance
- Implemented
  - Discovery of `.py` scripts (project `.pyst/` and global dirs)
  - Execution via `uv run` with real-time stdout/stderr
  - Argument forwarding (with consistent `--` handling in dry-run and execution)
  - Working directory policy (project/script/current/custom)
  - Context rules (last-match-wins, with negation)
  - Introspection (safe AST mode) + JSON schema + caching with --no-cache
  - Offline mode support with proper CLI override propagation
  - Exit code mapping (102 for network-required failures in offline mode)
  - Cache validation using correct Python environment (uv-managed)
  - Basic install/uninstall/update (GitHub repo/gist/raw URL) + manifest
  - JSON output for `list`; `info`, `which`, `explain`
  - Shell completions (bash, zsh)
  - MCP server over stdio (text content responses)
- In progress
  - More robust argparse/click/typer introspection and batch mode
  - Context “extends” and provenance in `explain`
  - PATH shims for installed scripts
  - MCP structured JSON responses; TCP transport
  - Document command provider abstraction (beyond current DSPy helper)
- Planned
  - Rich CLI parameter discovery across frameworks
  - Markdown output tables for easy docs export

Contents
- Quick start
- Installation
- Usage
- Configuration
- Contexts
- Installing scripts
- MCP (Model Context Protocol)
- Testing
- Exit codes
- Troubleshooting
- Contributing
- License
- Acknowledgments

Quick start
- Create a `.pyst/` directory in your project and add a script:
  ```python
  # .pyst/hello.py
  #!/usr/bin/env python3
  """Say hello from pyst."""
  # /// script
  # requires-python = ">=3.8"
  # ///

  if __name__ == "__main__":
      print("Hello from pyst!")
  ```
- List and run:
  ```bash
  pyst list
  pyst run hello
  ```

Installation

Requirements
- Rust (for building)
- uv on PATH (for Python execution)
- Python 3.8+ available (uv can manage interpreters when online)

Install pyst
- From the repo:
  ```bash
  cargo install --path .    # from a checked-out workspace
  ```
- Or from Git:
  ```bash
  cargo install --git https://github.com/yourusername/pyst
  ```

Usage

Common commands
- Discover and run
  ```bash
  pyst list
  pyst run <name> [-- ARGS...]
  pyst which <name>
  pyst info <name>
  pyst explain <name> [--format json]
  ```
- Formats and completions
  ```bash
  pyst list --format json
  pyst completions bash | sudo tee /etc/bash_completion.d/pyst
  ```
- Cache control
  ```bash
  pyst cache path
  pyst cache clear
  ```
- Context control
  ```bash
  PYST_CONTEXT=default pyst list
  pyst run <name> --force
  ```

Selected flags
- Global
  - `--context <CTX>`: set active context (also respects `PYST_CONTEXT`)
  - `--config <PATH>`: use a specific `pyst.toml`
  - `--no-cache`: bypass introspection cache for this run
  - `--offline`: set `UV_NO_NETWORK=1` for child processes (uv)
  - `--cwd <PATH>`: override working directory for script execution
  - `--uv-flags "<FLAGS>"`: pass additional flags to uv (CLI > env `PYST_UV_FLAGS` > config)
  - `--no-color`, `--color=auto|always|never`, `-v/--verbose`, `-q/--quiet` (reserved; minimal logging today)
- Run-specific
  - `--force`: bypass context rules
  - `--dry-run`: print the uv command, cwd, env deltas

Configuration

Where
- Project: `./.pyst.toml` (nearest ancestor)
- Global: `~/.config/pyst/pyst.toml` (XDG)
- Env overrides: see below

Example `.pyst.toml`
```toml
[core]
project_script_dir = ".pyst"
global_script_dirs = ["~/.local/share/pyst/scripts"]
precedence = "local"        # local wins vs global
offline = false             # allow network by default
cwd = "project"             # "project" | "script" | "current" | "/abs/path"
introspection = "safe"      # "safe" | "import" (import mode only for trusted)

[core.uv]
flags = ["--python-preference=managed"]

[document]
# Current helper uses DSPy + OpenRouter model (optional feature)
model = "google/gemini-2.5-flash"
api_key_env = "OPENROUTER_API_KEY"
api_base = "https://openrouter.ai/api/v1"
max_tokens = 150
temperature = 0.7
redact = ["SECRET_*", "API_KEY_*", "PASSWORD_*"]

[contexts]
  [contexts.default]
  enabled = ["*", "!db-*", "!deploy-prod"]
```

Environment overrides
- `PYST_CONTEXT`, `PYST_OFFLINE`, `PYST_PRECEDENCE`, `PYST_INTROSPECTION`
- `PYST_PROJECT_SCRIPT_DIR`, `PYST_GLOBAL_SCRIPT_DIRS` (colon-separated)
- `PYST_UV_FLAGS` (space-separated)

Working directory policy
- `project` (default): nearest ancestor with `.pyst.toml` else `.git` else current dir
- `script`: directory of the script
- `current`: do not change cwd
- custom path: absolute path

Contexts

Behavior
- Rules are glob patterns; last match wins. Use `!pattern` to disable.
- Applied uniformly to named and path-based runs; `--force` bypasses.

Examples
```bash
# default context from file or env
pyst list

# Run disabled script with force
pyst run db-backup --force

# Explain a decision
pyst explain db-backup --format json
```

Installing scripts

Sources supported
- GitHub repo: clones the repo and installs all `.py` files (or a specific `blob/.../file.py`)
- GitHub Gist: installs `.py` files from a gist
- Raw URL: installs a single `.py` file

Examples
```bash
pyst install https://github.com/user/repo
pyst install https://github.com/user/repo/blob/main/tools/hello.py --as greet
pyst install https://gist.github.com/user/abc123
pyst install https://example.com/script.py
pyst list --all
pyst update greet
pyst uninstall greet
```

Notes
- Installed scripts are treated as global scripts and tracked in a manifest (JSON).
- If a local script shares the same name, precedence rules apply.
- PATH shims for direct execution are planned but not implemented yet; use `pyst run <name>`.

MCP (Model Context Protocol)

- Start server (stdio transport)
  ```bash
  pyst mcp
  ```
- Tools exposed:
  - `list_scripts`
  - `run_script`
  - `get_script_info`
  - `explain_script`
- Responses currently contain human-readable text content; structured JSON responses are planned.

Testing

- Native
  ```bash
  cargo test --lib --bins
  cargo test --test integration_tests
  ```
- Container-based (Linux; requires Docker)
  - A prebuilt test image with uv/Rust/Python speeds up integration tests
  ```bash
  # Build the optimized image
  cd pyst/tests/containers
  docker build -t pyst-test:latest .

  # Back to project root
  cargo test --package pyst containers -- --nocapture
  cargo test --package pyst integration -- --nocapture
  cargo test --package pyst test_optimized_pyst_workflow -- --nocapture
  ```

Exit codes

- 0: success
- 1: generic error (e.g., script runtime error)
- 64: CLI usage error
- 101: blocked by context/policy
- 102: network required but offline mode is active
- 127: script not found

Troubleshooting

- uv not found
  - Install uv and ensure it’s on PATH: https://astral.sh/uv/
- Offline failures
  - When `--offline` (or `PYST_OFFLINE=1`) is set and dependencies are not hydrated, runs will fail with exit code 102. Re-run without offline to allow uv to resolve environments.
  - The `--offline` flag now properly affects all operations including introspection and caching.
- Nothing listed
  - Ensure scripts exist in `.pyst/` (project) or in your configured global dirs.
- Duplicate entries in list
  - If a script exists both locally and installed globally, it may appear twice in `--all`. Resolution and execution follow precedence settings.

Contributing

- Build
  ```bash
  cargo build
  ```
- Run local tests (see Testing)
- PRs welcome! Please keep the README accurate to implementation status.

License

MIT License. See `LICENSE`.

Acknowledgments

- uv for fast Python dependency management
- Python packaging community and PEP 723 for inline script metadata
- README structure informed by community templates and guides:
  - [github.com](https://github.com/othneildrew/Best-README-Template)
  - [freecodecamp.org](https://www.freecodecamp.org/news/how-to-write-a-good-readme-file/)
  - [forum.cursor.com](https://forum.cursor.com/t/any-tips-on-creating-a-readme-md-from-within-cursor/6716)

Notes for maintainers
- Keep this README aligned with actual behavior. When promoting a feature from “In progress” to “Implemented,” add a minimal example snippet and, where relevant, a link to tests or docs.