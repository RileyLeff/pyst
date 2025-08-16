#!/usr/bin/env python3
"""Test working directory in containers."""
# /// script
# requires-python = ">=3.8"
# ///

import os
import sys

def main():
    cwd = os.getcwd()
    print(f"Container working directory: {cwd}")
    
    # Write output file to verify working directory
    with open("container_cwd_test.txt", "w") as f:
        f.write(f"Container script ran from: {cwd}\n")
    
    print(f"Created test file: {os.path.abspath('container_cwd_test.txt')}")

if __name__ == "__main__":
    main()