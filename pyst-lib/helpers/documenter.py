#!/usr/bin/env python3
"""
Documentation generator for pyst using DSPy.

This helper script generates concise, informative descriptions for Python scripts
using AI models through the DSPy framework.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "dspy",
# ]
# ///

import json
import sys
import os
from typing import Optional, Dict, Any
import dspy


class DocumentationSignature(dspy.Signature):
    """Generate a concise, informative description for a Python script."""
    
    script_content: str = dspy.InputField(desc="The complete Python script content")
    entry_point: str = dspy.InputField(desc="Entry point type (PEP723, Framework, MainFunction, etc.)")
    functions: str = dspy.InputField(desc="JSON list of function names and docstrings")
    dependencies: str = dspy.InputField(desc="JSON list of dependencies")
    current_description: str = dspy.InputField(desc="Current description if any, or empty string")
    max_length: int = dspy.InputField(desc="Maximum character length for description")
    
    description: str = dspy.OutputField(
        desc="A concise, informative description that explains what the script does (not how). "
             "Should be suitable for command discovery and fit terminal display. "
             "Focus on the script's purpose and main functionality."
    )


class ScriptDocumenter(dspy.Module):
    """DSPy module for generating script documentation."""
    
    def __init__(self):
        super().__init__()
        self.generate_description = dspy.ChainOfThought(DocumentationSignature)
    
    def forward(self, script_content: str, entry_point: str, functions: str, 
                dependencies: str, current_description: str = "", max_length: int = 80):
        """Generate a description for the given script."""
        
        result = self.generate_description(
            script_content=script_content,
            entry_point=entry_point,
            functions=functions,
            dependencies=dependencies,
            current_description=current_description,
            max_length=max_length
        )
        
        # Ensure the description fits within the length limit
        description = result.description.strip()
        if len(description) > max_length:
            # Try to truncate at word boundary
            words = description[:max_length].split()
            if len(words) > 1:
                words.pop()  # Remove last potentially truncated word
                description = " ".join(words) + "..."
            else:
                description = description[:max_length-3] + "..."
        
        return description


def configure_dspy(config: Dict[str, Any]) -> None:
    """Configure DSPy with the provided settings."""
    
    # Extract configuration
    model = config.get("model", "google/gemini-2.5-flash")
    api_key_env = config.get("api_key_env", "OPENROUTER_API_KEY")
    api_base = config.get("api_base", "https://openrouter.ai/api/v1")
    max_tokens = config.get("max_tokens")
    temperature = config.get("temperature")
    
    # Get API key from environment
    api_key = os.getenv(api_key_env)
    if not api_key:
        raise ValueError(f"API key not found in environment variable: {api_key_env}")
    
    # Configure LM with optional parameters
    lm_kwargs = {
        "api_key": api_key,
    }
    
    if api_base:
        lm_kwargs["api_base"] = api_base
    if max_tokens:
        lm_kwargs["max_tokens"] = max_tokens
    if temperature is not None:
        lm_kwargs["temperature"] = temperature
    
    # For OpenRouter, we need to specify the model in the OpenRouter format
    # and set the custom_llm_provider
    if "openrouter.ai" in str(api_base):
        lm_kwargs["custom_llm_provider"] = "openrouter"
        # Keep the model as-is for OpenRouter
    
    # Configure DSPy
    lm = dspy.LM(model, **lm_kwargs)
    dspy.configure(lm=lm)


def generate_documentation(script_data: Dict[str, Any], config: Dict[str, Any]) -> Dict[str, Any]:
    """Generate documentation for a script using DSPy."""
    
    try:
        # Configure DSPy
        configure_dspy(config)
        
        # Create documenter
        documenter = ScriptDocumenter()
        
        # Extract data
        script_content = script_data.get("script_content", "")
        entry_point = script_data.get("entry_point", "Unknown")
        functions = json.dumps(script_data.get("functions", []))
        dependencies = json.dumps(script_data.get("dependencies", []))
        current_description = script_data.get("current_description", "")
        max_length = script_data.get("max_length", 80)
        
        # Generate description
        description = documenter.forward(
            script_content=script_content,
            entry_point=entry_point,
            functions=functions,
            dependencies=dependencies,
            current_description=current_description,
            max_length=max_length
        )
        
        return {
            "success": True,
            "description": description,
            "error": None
        }
        
    except Exception as e:
        return {
            "success": False,
            "description": None,
            "error": str(e)
        }


def main():
    """Main entry point for the documenter script."""
    
    if len(sys.argv) != 2:
        print("Usage: documenter.py <config_and_data_json>")
        sys.exit(1)
    
    try:
        # Parse input JSON
        input_data = json.loads(sys.argv[1])
        
        # Extract configuration and script data
        config = input_data.get("config", {})
        script_data = input_data.get("script_data", {})
        
        # Generate documentation
        result = generate_documentation(script_data, config)
        
        # Output result as JSON
        print(json.dumps(result))
        
    except json.JSONDecodeError as e:
        result = {
            "success": False,
            "description": None,
            "error": f"Invalid JSON input: {e}"
        }
        print(json.dumps(result))
        sys.exit(1)
    
    except Exception as e:
        result = {
            "success": False,
            "description": None,
            "error": f"Unexpected error: {e}"
        }
        print(json.dumps(result))
        sys.exit(1)


if __name__ == "__main__":
    main()