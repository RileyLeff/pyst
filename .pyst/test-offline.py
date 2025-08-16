#!/usr/bin/env python3
"""Test script for offline mode functionality."""
# /// script
# requires-python = ">=3.8"
# dependencies = ["requests>=2.28.0"]
# ///

import requests

def main():
    print("This script requires requests, which might not be installed")
    response = requests.get("https://httpbin.org/json")
    print(f"Response: {response.json()}")

if __name__ == "__main__":
    main()