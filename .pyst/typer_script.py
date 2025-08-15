"""Greets the user by name, defaulting to "World"."""
# /// script
# dependencies = ["typer"]
# ///

import typer

def main(name: str = "World"):
    print(f"Hello {name}!")

if __name__ == "__main__":
    typer.run(main)