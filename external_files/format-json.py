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