#!/usr/bin/env python3
"""Test exit codes are properly bubbled up."""
# /// script
# requires-python = ">=3.8"
# ///

import sys

def main():
    if len(sys.argv) > 1:
        exit_code = int(sys.argv[1])
        print(f"Exiting with code: {exit_code}")
        sys.exit(exit_code)
    else:
        print("Usage: test-exit-codes.py <exit_code>")
        print("Example: test-exit-codes.py 5")
        sys.exit(0)

if __name__ == "__main__":
    main()