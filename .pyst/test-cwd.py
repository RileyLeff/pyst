#!/usr/bin/env python3
"""Test working directory functionality."""
# /// script
# requires-python = ">=3.8"
# ///

import os
import sys

def main():
    cwd = os.getcwd()
    print(f"Current working directory: {cwd}")
    
    # Write a test file to verify we're in the right directory
    test_file = "cwd_test_output.txt"
    with open(test_file, "w") as f:
        f.write(f"Script ran from: {cwd}\n")
    
    print(f"Created test file: {os.path.abspath(test_file)}")

if __name__ == "__main__":
    main()