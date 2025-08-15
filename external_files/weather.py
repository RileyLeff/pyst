#!/usr/bin/env python3
"""
Weather information script using httpx.

Fetches weather data for a given city.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "httpx>=0.24.0",
#     "typer>=0.9.0",
#     "rich>=13.0.0"
# ]
# ///

import typer
import httpx
from rich.console import Console
from rich.table import Table

app = typer.Typer()
console = Console()

@app.command()
def main(city: str = typer.Argument(..., help="City name")):
    """Get weather information for a city."""
    try:
        # Mock weather API call
        response = httpx.get(f"https://api.example.com/weather/{city}")
        console.print(f"[blue]Weather for {city}:[/blue] Sunny, 75°F")
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")

if __name__ == "__main__":
    app()