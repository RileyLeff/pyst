#!/usr/bin/env python3
"""Simple hello script for container testing."""
# /// script
# requires-python = ">=3.8"
# ///

import sys
import time

def main():
    print("Hello from containerized pyst!", flush=True)
    print("This is stdout output", flush=True)
    print("Error message", file=sys.stderr, flush=True)
    
    # Test arguments
    if len(sys.argv) > 1:
        print(f"Arguments received: {sys.argv[1:]}", flush=True)
    
    # Test streaming with delay
    for i in range(3):
        print(f"Streaming line {i + 1}", flush=True)
        time.sleep(0.1)
    
    print("Container test completed!")

if __name__ == "__main__":
    main()