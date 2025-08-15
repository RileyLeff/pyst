# Test Files for Pyst Phase 3 Implementation

Please create the following external resources for testing the `pyst install` functionality:

## 1. GitHub Repository: "pyst-test-scripts"

Create a public GitHub repo with these files:

### `hello.py` (Simple script with PEP 723)
```python
#!/usr/bin/env python3
"""
A simple greeting script with dependencies.

Usage: hello.py [name]
"""
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "click>=8.0.0",
#     "rich>=13.0.0"
# ]
# ///

import click
from rich.console import Console

console = Console()

@click.command()
@click.argument('name', default='World')
def main(name: str):
    """Say hello with style using Rich."""
    console.print(f"[bold green]Hello, {name}![/bold green] 🎉")
    console.print("[dim]Powered by pyst[/dim]")

if __name__ == "__main__":
    main()
```

### `weather.py` (More complex script)
```python
#!/usr/bin/env python3
"""
Weather information script using httpx.

Fetches weather data for a given city.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "httpx>=0.24.0",
#     "typer>=0.9.0",
#     "rich>=13.0.0"
# ]
# ///

import typer
import httpx
from rich.console import Console
from rich.table import Table

app = typer.Typer()
console = Console()

@app.command()
def main(city: str = typer.Argument(..., help="City name")):
    """Get weather information for a city."""
    try:
        # Mock weather API call
        response = httpx.get(f"https://api.example.com/weather/{city}")
        console.print(f"[blue]Weather for {city}:[/blue] Sunny, 75°F")
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")

if __name__ == "__main__":
    app()
```

### `db-backup.py` (Script that should be disabled by context)
```python
#!/usr/bin/env python3
"""
Database backup utility.

This script should be disabled by default context rules.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "psycopg2-binary>=2.9.0"
# ]
# ///

def main():
    """Backup database (mock implementation)."""
    print("🗄️ Starting database backup...")
    print("✅ Backup completed successfully")

if __name__ == "__main__":
    main()
```

## 2. GitHub Gist: Single Script

Create a public gist with this file:

### `format-json.py` (JSON formatter utility)
```python
#!/usr/bin/env python3
"""
JSON formatting utility.

Formats JSON files with proper indentation.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

import json
import sys
from pathlib import Path

def main():
    """Format JSON files."""
    if len(sys.argv) != 2:
        print("Usage: format-json.py <file.json>")
        sys.exit(1)
    
    file_path = Path(sys.argv[1])
    
    if not file_path.exists():
        print(f"Error: {file_path} not found")
        sys.exit(1)
    
    try:
        with open(file_path) as f:
            data = json.load(f)
        
        with open(file_path, 'w') as f:
            json.dump(data, f, indent=2, sort_keys=True)
        
        print(f"✅ Formatted {file_path}")
    except json.JSONDecodeError as e:
        print(f"❌ Invalid JSON: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## 3. Raw URL Script

Host this file somewhere accessible via direct URL (GitHub raw, your own server, etc.):

### `uuid-gen.py` (Simple UUID generator)
```python
#!/usr/bin/env python3
"""
UUID generator utility.

Generates various types of UUIDs.
"""
import uuid
import sys

def main():
    """Generate a UUID."""
    uuid_type = sys.argv[1] if len(sys.argv) > 1 else "4"
    
    if uuid_type == "1":
        result = uuid.uuid1()
    elif uuid_type == "4":
        result = uuid.uuid4()
    else:
        print("Usage: uuid-gen.py [1|4]")
        sys.exit(1)
    
    print(result)

if __name__ == "__main__":
    main()
```

## Test URLs Needed

Once created, please provide:

1. **GitHub repo URL**: `https://github.com/yourusername/pyst-test-scripts`
2. **Gist URL**: `https://gist.github.com/yourusername/[gist-id]`
3. **Raw URL**: Direct link to `uuid-gen.py` file

## Test Commands to Implement

These will be the test cases for Phase 3:

```bash
# Install from GitHub repo (should install all 3 scripts)
pyst install https://github.com/yourusername/pyst-test-scripts

# Install specific script with custom name
pyst install https://github.com/yourusername/pyst-test-scripts/blob/main/hello.py --as greet

# Install from gist
pyst install https://gist.github.com/yourusername/[gist-id]

# Install from raw URL
pyst install https://raw.githubusercontent.com/yourusername/pyst-test-scripts/main/uuid-gen.py

# List should show install sources
pyst list --all

# Update and uninstall
pyst update hello
pyst uninstall weather
```

Let me know when these are ready and I'll proceed with Phase 3 implementation!