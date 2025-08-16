#!/usr/bin/env python3
"""Test CLI framework argument forwarding in containers."""
# /// script
# requires-python = ">=3.8"
# dependencies = ["click>=8.0.0"]
# ///

import click
import sys

@click.command()
@click.option('--name', '-n', default='Container', help='Name to greet')
@click.option('--count', '-c', default=1, type=int, help='Number of greetings')
@click.option('--verbose', '-v', is_flag=True, help='Verbose output')
@click.argument('extra_args', nargs=-1)
def main(name, count, verbose, extra_args):
    """Test script for Click argument forwarding in containers."""
    
    if verbose:
        print("Container verbose mode enabled")
        print(f"Container raw sys.argv: {sys.argv}")
    
    for i in range(count):
        print(f"Container hello, {name}! (greeting {i + 1})")
    
    if extra_args:
        print(f"Container extra arguments: {list(extra_args)}")
    
    print("Container Click test completed")

if __name__ == "__main__":
    main()