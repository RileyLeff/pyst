#!/usr/bin/env python3
"""Test complex argument forwarding."""
# /// script
# requires-python = ">=3.8"
# dependencies = ["click>=8.0.0"]
# ///

import click
import sys

@click.command()
@click.option('--name', '-n', default='World', help='Name to greet')
@click.option('--count', '-c', default=1, type=int, help='Number of greetings')
@click.option('--verbose', '-v', is_flag=True, help='Verbose output')
@click.argument('extra_args', nargs=-1)
def main(name, count, verbose, extra_args):
    """Test script for argument forwarding with click."""
    
    if verbose:
        print("Verbose mode enabled")
        print(f"Raw sys.argv: {sys.argv}")
    
    for i in range(count):
        print(f"Hello, {name}! (greeting {i + 1})")
    
    if extra_args:
        print(f"Extra arguments: {list(extra_args)}")
    
    # Test edge cases
    print("Testing edge cases:")
    print("  - Spaces in arguments")
    print("  - Special characters: !@#$%^&*()")
    print("  - Unicode: 你好世界 🌍")

if __name__ == "__main__":
    main()