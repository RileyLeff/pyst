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