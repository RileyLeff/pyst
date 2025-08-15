# pyst 🐍⚡

A modern, ergonomic command runner for the Python ecosystem. Transform your scattered Python scripts into a coherent, discoverable, and shareable toolset.

```bash
# Discover and run scripts effortlessly
pyst list                    # See all available scripts
pyst run hello               # Execute with automatic dependency management
pyst install github.com/user/repo  # Share scripts across projects
pyst document script.py      # AI-powered documentation generation
```

## 🌟 Features

### 🔍 **Intelligent Script Discovery**
- Automatic detection of Python scripts in your project and globally
- Smart entry point detection (PEP 723, Click/Typer, `main()` functions)
- Respects `.pyst` project directories and `~/.local/share/pyst/scripts` global scripts

### ⚡ **Zero-Config Execution**
- Leverages `uv` for blazingly fast dependency resolution and execution
- PEP 723 support for inline script metadata
- Automatic virtual environment management
- Works with existing `pyproject.toml` configurations

### 🎯 **Context-Aware Filtering**
- Rule-based script enabling/disabling with glob patterns
- Environment-specific contexts (development, production, etc.)
- `--force` flag to bypass restrictions when needed

### 🌐 **Sharing Ecosystem**
- Install scripts from GitHub repositories, Gists, or raw URLs
- Commit pinning for reproducible installations
- SHA256 integrity verification
- Simple update and uninstall management

### 🤖 **AI-Powered Documentation**
- Powered by DSPy framework with configurable LLM providers
- Generates concise, terminal-friendly descriptions
- Interactive approval workflow with diff preview
- Guardrails with `.pystdocignore` and inline markers

### 🛡️ **Security & Trust**
- Safe AST-based introspection by default
- Opt-in import-mode for trusted scripts
- Sensitive data redaction in configurations
- Sandboxed subprocess execution

## 🚀 Quick Start

### Installation

```bash
# Install pyst (requires Rust and uv)
cargo install --git https://github.com/yourusername/pyst
```

### Basic Usage

```bash
# List available scripts
pyst list

# Run a script
pyst run my-script

# Get detailed information about a script
pyst info my-script

# Check why a script is enabled/disabled
pyst explain my-script
```

### Install Scripts from the Web

```bash
# Install from GitHub repository
pyst install https://github.com/user/awesome-scripts

# Install from Gist
pyst install https://gist.github.com/user/abc123

# Install from raw URL with custom name
pyst install https://example.com/script.py --as my-tool

# Manage installed scripts
pyst update my-tool
pyst uninstall my-tool
```

### AI Documentation Generation

```bash
# Set up your API key (OpenRouter + Gemini 2.5 Flash by default)
export OPENROUTER_API_KEY=your-key-here

# Generate documentation interactively
pyst document my-script.py

# Write documentation directly
pyst document my-script.py --write

# Check if documentation exists
pyst document my-script.py --check
```

## 📁 Project Structure

```
your-project/
├── .pyst.toml              # Project configuration
├── .pyst/                  # Local scripts directory
│   ├── deploy.py          # Deployment script
│   ├── test-runner.py     # Custom test runner
│   └── .pystdocignore     # Documentation ignore rules
├── scripts/               # Alternative scripts location
└── pyproject.toml         # Standard Python project config
```

## ⚙️ Configuration

Create a `.pyst.toml` file in your project root or globally at `~/.config/pyst/pyst.toml`:

```toml
[core]
# Script discovery locations
global_script_dirs = ["~/.local/share/pyst/scripts"]
project_script_dir = ".pyst"
precedence = "local"  # local scripts override global ones

# Execution settings
cwd = "project"      # Run from project root
introspection = "safe"  # Use AST parsing by default
offline = false      # Allow network access

[core.uv]
flags = ["--python-preference=managed"]

[document]
# AI documentation settings (OpenRouter + Gemini 2.5 Flash)
model = "google/gemini-2.5-flash"
api_key_env = "OPENROUTER_API_KEY"
api_base = "https://openrouter.ai/api/v1"
max_tokens = 150
temperature = 0.7
redact = ["SECRET_*", "API_KEY_*"]

# Alternative providers:
# For OpenAI: model = "gpt-4", api_base = "https://api.openai.com/v1"
# For Anthropic: model = "claude-3-haiku-20240307"

[contexts]
  [contexts.default]
  enabled = ["*", "!test-*", "!deploy-prod"]
  
  [contexts.production]
  enabled = ["deploy-*", "monitor-*"]
```

## 📝 Script Examples

### PEP 723 Script
```python
#!/usr/bin/env python3
"""Database backup utility with progress tracking."""
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "psycopg2-binary>=2.9.0",
#     "rich>=13.0.0"
# ]
# ///

import psycopg2
from rich.progress import track

def main():
    # Your backup logic here
    for table in track(tables, description="Backing up..."):
        backup_table(table)

if __name__ == "__main__":
    main()
```

### Typer CLI Script
```python
#!/usr/bin/env python3
"""Git repository analyzer with detailed statistics."""
# /// script
# dependencies = ["typer>=0.9.0", "pygit2>=1.13.0"]
# ///

import typer
from pathlib import Path

app = typer.Typer()

@app.command()
def analyze(repo_path: Path = typer.Argument(..., help="Path to Git repository")):
    """Analyze Git repository and show statistics."""
    typer.echo(f"Analyzing repository: {repo_path}")
    # Analysis logic here

if __name__ == "__main__":
    app()
```

## 🎭 Context Management

Control script execution with powerful context rules:

```bash
# Set active context
export PYST_CONTEXT=production

# Only deployment scripts will be enabled
pyst list

# Override context rules temporarily
pyst run test-script --force

# Understand why a script is disabled
pyst explain test-script
```

## 🔒 Security & Trust

### Safe Introspection (Default)
```bash
# Uses AST parsing - secure but limited
pyst info untrusted-script.py
```

### Trusted Script Execution
```bash
# Mark scripts/directories as trusted for full introspection
pyst trust ~/.local/share/pyst/scripts
pyst trust ./my-script.py

# Now import-mode introspection is available
pyst info my-script.py  # Shows runtime information
```

### Documentation Guardrails

Create `.pystdocignore` to exclude files:
```
# Ignore test files
test_*.py
*_test.py

# Ignore sensitive scripts
secrets/*.py
deploy/prod-*.py
```

Or use inline markers:
```python
#!/usr/bin/env python3
# pyst:doc:ignore
"""This script won't be documented."""
```

## 🔧 Advanced Usage

### JSON Output for Automation
```bash
# Machine-readable script listing
pyst list --format json | jq '.[] | select(.enabled)'

# Integration with other tools
pyst list --format json | tools/analyze-scripts.py
```

### Cache Management
```bash
# Clear introspection cache
pyst cache clear

# Show cache location
pyst cache path
```

### Shell Completions
```bash
# Generate completions for your shell
pyst completions bash > ~/.local/share/bash-completion/completions/pyst
pyst completions zsh > ~/.local/share/zsh/site-functions/_pyst
```

## 🏗️ Architecture

**pyst** follows a clean, modular architecture:

- **Discovery Engine**: Finds and catalogs Python scripts
- **Introspection System**: Analyzes scripts safely using AST or import modes
- **Context Engine**: Applies rule-based filtering
- **Execution Engine**: Runs scripts via `uv` with proper isolation
- **Installation Manager**: Handles remote script installation and updates
- **Documentation AI**: Generates descriptions using DSPy + LLMs

### Key Design Principles

1. **Zero Configuration**: Works out of the box with sensible defaults
2. **Performance**: Leverages `uv` for fast dependency resolution
3. **Security**: Safe by default with opt-in trust mechanisms
4. **Extensibility**: Plugin-ready architecture for future enhancements
5. **AI Integration**: Modern tooling enhanced with LLM capabilities

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/pyst
cd pyst

# Build the project
cargo build

# Run tests
cargo test

# Test with real scripts
./target/debug/pyst list
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [uv](https://github.com/astral-sh/uv) - Blazingly fast Python package installer
- [DSPy](https://github.com/stanfordnlp/dspy) - Programming language models framework
- [PEP 723](https://peps.python.org/pep-0723/) - Inline script metadata standard
- [Rust](https://rust-lang.org) - Systems programming language for performance and safety

---

**pyst**: From scattered scripts to sophisticated toolchains. 🐍⚡