Of course. This is the definitive blueprint for `pyst`, incorporating all of our design decisions, the expert AI-driven feedback, and the final resolutions. This document is a complete and actionable technical plan, ready for implementation.

---

## **`pyst`: Final Project Description & Implementation Plan**

### **1. Project Description & Vision**

`pyst` is a modern, ergonomic command runner for the Python ecosystem, inspired by the simplicity and discoverability of `just`. Its purpose is to elevate individual Python scripts into first-class, self-documenting command-line tools.

`pyst` acts as a unified management plane for a developer's entire arsenal of local and global scripts. It leverages the speed of `uv` for execution and the clarity of PEP 723 for dependency management, providing a seamless, high-performance experience. By automating discovery, documentation, and execution, `pyst` transforms scattered `.py` files into a coherent, shareable, and secure toolset for both human developers and AI agents.

### **2. Core Concepts & Rules**

*   **Configuration:** A cascading system with a clear precedence: **Environment Variables > Project-Local `.pyst.toml` > Global `pyst.toml` > Built-in Defaults**. Maps are merged recursively; arrays are **replaced** by higher-precedence sources.
*   **Project Root Detection:** The project root is determined by the first match in this order: 1) Nearest ancestor directory with a `.pyst.toml`, 2) Nearest ancestor with a `.git`, 3) The current working directory.
*   **Script Discovery:** A "best-effort" discovery of runnable `.py` files (ignoring `_*`/`.pyi` patterns), prioritized by the first entry point found: 1) PEP 723 script metadata block, 2) Recognized framework object (`Typer`/`Click`), 3) `def main(...)` function.
*   **Name Resolution:** Script names are resolved with clear precedence (`local` vs. `global` set in config). Explicit selectors (`project:script`, `global:script`) and direct path execution are supported.
*   **Contexts:** Named execution profiles that act as application-layer sandboxes.
    *   **Activation:** Active context is determined by: `--context` flag > `PYST_CONTEXT` env var > `default` context.
    *   **Rules:** An `enabled` list of glob patterns is evaluated in order; the **last matching rule wins**.
    *   **Uniformity:** Contexts are applied to all script executions, including those invoked by absolute path, unless `--force` is used.

### **3. Complete Command-Line Interface (CLI)**

#### **Global Flags**
*   `--context <CTX>`, `--config <PATH>`, `--no-cache`, `--no-color`, `--color=<auto|always|never>`.
*   `--offline`: Disallow network access for dependency resolution.
*   `--cwd <PATH>`: Override the default execution directory (project root).
*   `--uv-flags "<FLAGS>"`: Pass additional flags directly to the underlying `uv` command.
*   `-v, --verbose`, `-q, --quiet`.

#### **Main Commands**
*   `pyst` (alias: `list`): The default discovery command.
*   `pyst list [--all] [--format <FMT>]`: Lists scripts. `--all` shows disabled scripts. `--format` supports `human`, `json`, `markdown`.
*   `pyst run [--force] [--dry-run] <script> [-- SCRIPT_ARGS...]`: Executes a script.
*   `pyst info <script>`: Displays detailed information for a single script.
*   `pyst which <script>`: Prints the absolute path to the resolved script file.
*   `pyst explain <script> [--format json]`: Explains why a script is enabled/disabled by context rules.
*   `pyst install <SOURCE> [--as <NAME>]`: Installs a script. Pins to a commit SHA.
*   `pyst uninstall <script>`: Uninstalls a managed script.
*   `pyst update <script>`: Updates an installed script to its latest version.
*   `pyst trust <script|dir>`: Marks a script or directory as trusted for privileged introspection.
*   `pyst document <script> [--write] [--check]`: Interactively generates documentation via an LLM.
*   `pyst completions <shell>`: Generates shell completion scripts.
*   `pyst cache <clear|path>`: Manages the introspection cache.
*   `pyst mcp [--port <PORT>] [--transport <stdio|tcp>]`: Starts the MCP server.

#### **Exit Code Semantics**
`pyst` will use distinct exit codes to communicate status to shell scripts and CI pipelines.
*   `0`: Success.
*   `1`: Generic error (e.g., script runtime error, file IO error).
*   `64`: CLI usage error (from `clap`).
*   `101`: Execution blocked by a context rule.
*   `102`: Execution failed due to network being required in `--offline` mode.
*   `127`: Script name not found.

### **4. Final `pyst.toml` Configuration**

```toml
[core]
global_script_dirs = ["~/.local/share/pyst/scripts"]
project_script_dir = ".pyst"
precedence = "local"
offline = false
cwd = "project" # Options: "project", "script", "current", "/abs/path"
introspection = "safe" # Options: "safe", "import"

[core.uv]
flags = ["--python-preference=managed"]

[document]
provider = "gemini"
model = "gemini-1.5-flash"
api_key_env = "GOOGLE_API_KEY"
redact = ["SECRET_*", "API_KEY_*"]

[contexts]
  [contexts.default]
  enabled = ["*", "!db-*", "!deploy-prod"]
```

### **5. System Architecture & Implementation Strategy**

#### **Project Structure: Cargo Workspace**
*   **`pyst-lib` (Core Engine):** A library crate containing all core, non-UI logic.
*   **`pyst` (CLI Binary):** A thin application crate that uses `pyst-lib` for logic and handles all CLI parsing and user-facing output.

#### **The Rust-Python Boundary: The Subprocess Model**
`pyst` is an **orchestrator**, not an interpreter. There will be **no PyO3/Maturin dependency**. The Rust binary will communicate with its Python helper scripts (`introspector.py`, `documenter.py`) by running them as sandboxed subprocesses using `uv run`.

#### **Core Crate (`pyst-lib`) Module Design**
*   **`config.rs`:** Defines the `pyst.toml` structure and implements the full cascading load logic.
*   **`discovery.rs`:** Implements script discovery, project root detection, and name resolution.
*   **`introspection/`:**
    *   **`cache.rs`:** Manages the cache file. Keyed by `hash(file_content + deps + python_version + schema_version)`.
    *   **`runner.rs`:** Implements the batched execution of `introspector.py` in a sandboxed process.
    *   **`schema.rs`:** Defines the stable, versioned JSON schema for introspection data.
*   **`executor/`:**
    *   **`context.rs`:** Implements context filtering, inheritance, and the logic for `pyst explain`.
    *   **`runner.rs`:** Implements `pyst run`, respecting project `[tool.uv]` config and constructing the final `uv` command.
*   **`install/`:** Implements `install`, `uninstall`, and `update`, including manifest management and PATH shim creation.
*   **`document/`:** Implements the `pyst document` logic, including respecting `.pystdocignore` files and inline markers for guardrails.
*   **`mcp.rs`:** Contains the `axum` server logic for the MCP.

#### **Testing Strategy**
*   **Rust CLI:** Use `assert_cmd` for integration tests and `insta` for snapshotting the formatted output of `list` and `info`.
*   **Python Helpers:** Use `pytest` for unit testing the `introspector.py` and `documenter.py` scripts against a variety of fixture files.
*   **CI:** The CI pipeline will run tests against a matrix of supported Python versions (e.g., 3.11, 3.12, 3.13) to ensure consistent behavior.

### **6. Phased Implementation Plan**

**Phase 1: Minimum Viable Product (The Core Runner)**
*   [ ] Implement CLI structure: `run`, `list`, `which`, `info`, `completions`.
*   [ ] Implement script discovery, project root detection, and explicit selectors.
*   [ ] Implement core execution via `uv run`, respecting a default project root CWD.
*   [ ] Set up the testing framework (`assert_cmd`, `pytest`).

**Phase 2: The Ergonomic Experience**
*   [ ] **Define and stabilize the JSON introspection schema (v1).**
*   [ ] Implement the full cascading `pyst.toml` system with XDG-correct paths.
*   [ ] Implement the dual-mode introspection engine (`safe`/`import`) and `pyst trust` command.
*   [ ] Implement the robust, hash-based caching system and `pyst cache` command.
*   [ ] Integrate introspection to create polished `list` and `info` output.
*   [ ] Implement the context system with `pyst explain` and the `--force` flag.

**Phase 3: The Sharing Ecosystem**
*   [ ] Implement `pyst install` with URL/GitHub/Gist support and commit-pinning.
*   [ ] Create the installation manifest with `sha256` integrity checks.
*   [ ] Implement `pyst uninstall` and `pyst update`.
*   [ ] Implement PATH shim creation and provide instructions to the user.

**Phase 4: The AI & Automation Frontier**
*   [ ] Implement `pyst list --format json`.
*   [ ] Implement the `pyst mcp` server (stdio transport first).
*   [ ] Develop the `documenter.py` helper with the provider abstraction.
*   [ ] Implement the `pyst document` command with its interactive diff-and-apply workflow and guardrails.