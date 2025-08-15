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