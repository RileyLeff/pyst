#!/usr/bin/env python3
"""
Pyst Introspector - A Python script for analyzing Python scripts.

This script runs in a sandboxed subprocess to safely introspect Python files
without affecting the main pyst process. It supports two modes:
- safe: Static analysis only (no imports)
- import: Dynamic analysis with imports (requires trust)
"""

import ast
import argparse
import json
import sys
import os
import re
import importlib.util
import hashlib
from pathlib import Path
from typing import Dict, List, Any, Optional, Union
import traceback


SCHEMA_VERSION = "1.0.0"


class SafeIntrospector:
    """Safe static analysis using AST parsing."""
    
    def __init__(self, script_path: Path):
        self.script_path = script_path
        self.content = script_path.read_text(encoding='utf-8')
        self.tree = None
        self.errors = []
        
    def introspect(self) -> Dict[str, Any]:
        """Perform static introspection."""
        try:
            self.tree = ast.parse(self.content, str(self.script_path))
        except SyntaxError as e:
            self.errors.append({
                "error_type": "SyntaxError",
                "message": str(e),
                "line_number": e.lineno
            })
            return self._create_error_result()
        
        try:
            metadata = {
                "name": self.script_path.stem,
                "path": str(self.script_path),
                "description": self._extract_description(),
                "docstring": self._extract_module_docstring(),
                "pep723_metadata": self._extract_pep723_metadata(),
                "dependencies": self._extract_dependencies(),
                "entry_points": self._extract_entry_points(),
                "functions": self._extract_functions(),
                "classes": self._extract_classes(),
                "imports": self._extract_imports(),
                "cli_framework": self._detect_cli_framework(),
                "errors": self.errors
            }
            
            return metadata
        except Exception as e:
            self.errors.append({
                "error_type": "RuntimeError",
                "message": f"Introspection failed: {str(e)}",
                "line_number": None
            })
            return self._create_error_result()
    
    def _create_error_result(self) -> Dict[str, Any]:
        """Create a minimal result when introspection fails."""
        return {
            "name": self.script_path.stem,
            "path": str(self.script_path),
            "description": None,
            "docstring": None,
            "pep723_metadata": None,
            "dependencies": [],
            "entry_points": [],
            "functions": [],
            "classes": [],
            "imports": [],
            "cli_framework": None,
            "errors": self.errors
        }
    
    def _extract_description(self) -> Optional[str]:
        """Extract description from docstring or comments."""
        docstring = self._extract_module_docstring()
        if docstring:
            # Get first line as description
            first_line = docstring.split('\n')[0].strip()
            if first_line and not first_line.startswith('"""') and not first_line.startswith("'''"):
                return first_line
        return None
    
    def _extract_module_docstring(self) -> Optional[str]:
        """Extract module-level docstring."""
        if self.tree and isinstance(self.tree.body[0], ast.Expr) and isinstance(self.tree.body[0].value, ast.Constant):
            return self.tree.body[0].value.value
        return None
    
    def _extract_pep723_metadata(self) -> Optional[Dict[str, Any]]:
        """Extract PEP 723 script metadata block."""
        lines = self.content.split('\n')
        in_script_block = False
        metadata_lines = []
        
        for line in lines:
            stripped = line.strip()
            if stripped == "# /// script":
                in_script_block = True
                continue
            elif stripped == "# ///" and in_script_block:
                break
            elif in_script_block:
                if stripped.startswith('#'):
                    metadata_lines.append(stripped[1:].strip())
        
        if metadata_lines:
            try:
                # Parse TOML-like metadata
                try:
                    import tomllib  # Python 3.11+
                except ImportError:
                    import tomli as tomllib  # fallback
                toml_content = '\n'.join(metadata_lines)
                metadata = tomllib.loads(toml_content)
                return {
                    "dependencies": metadata.get("dependencies", []),
                    "requires_python": metadata.get("requires-python"),
                    "tool_config": metadata.get("tool", {})
                }
            except Exception:
                # Fallback to simple parsing
                dependencies = []
                for line in metadata_lines:
                    if line.startswith('dependencies = '):
                        # Simple list parsing
                        deps_str = line.replace('dependencies = ', '').strip()
                        if deps_str.startswith('[') and deps_str.endswith(']'):
                            deps_str = deps_str[1:-1]
                            dependencies = [dep.strip(' "\'') for dep in deps_str.split(',') if dep.strip()]
                
                if dependencies:
                    return {
                        "dependencies": dependencies,
                        "requires_python": None,
                        "tool_config": {}
                    }
        
        return None
    
    def _extract_dependencies(self) -> List[Dict[str, Any]]:
        """Extract dependencies from various sources."""
        dependencies = []
        
        # From PEP 723 metadata
        pep723 = self._extract_pep723_metadata()
        if pep723:
            for dep in pep723["dependencies"]:
                dependencies.append({
                    "name": dep.split("==")[0].split(">=")[0].split("<=")[0].split(">")[0].split("<")[0],
                    "version_spec": dep.replace(dep.split("==")[0].split(">=")[0].split("<=")[0].split(">")[0].split("<")[0], "") or None,
                    "source": "Pep723"
                })
        
        # From imports (approximation)
        for import_info in self._extract_imports():
            if not import_info["module"].startswith('.') and '.' not in import_info["module"]:
                # Likely third-party package
                dependencies.append({
                    "name": import_info["module"],
                    "version_spec": None,
                    "source": "Import"
                })
        
        return dependencies
    
    def _extract_entry_points(self) -> List[Dict[str, Any]]:
        """Extract potential entry points."""
        entry_points = []
        
        if not self.tree:
            return entry_points
        
        for node in ast.walk(self.tree):
            if isinstance(node, ast.FunctionDef):
                if node.name == "main":
                    entry_points.append({
                        "name": "main",
                        "callable": "main",
                        "module": None,
                        "entry_type": "Main"
                    })
                elif any(decorator.id == "click.command" if isinstance(decorator, ast.Attribute) else False for decorator in node.decorator_list):
                    entry_points.append({
                        "name": node.name,
                        "callable": node.name,
                        "module": None,
                        "entry_type": "CliCommand"
                    })
        
        return entry_points
    
    def _extract_functions(self) -> List[Dict[str, Any]]:
        """Extract function information."""
        functions = []
        
        if not self.tree:
            return functions
        
        for node in ast.walk(self.tree):
            if isinstance(node, ast.FunctionDef):
                func_info = {
                    "name": node.name,
                    "line_number": node.lineno,
                    "docstring": ast.get_docstring(node),
                    "parameters": self._extract_parameters(node),
                    "returns": self._extract_return_annotation(node),
                    "decorators": [self._decorator_to_string(d) for d in node.decorator_list],
                    "is_async": isinstance(node, ast.AsyncFunctionDef)
                }
                functions.append(func_info)
        
        return functions
    
    def _extract_parameters(self, func_node: ast.FunctionDef) -> List[Dict[str, Any]]:
        """Extract function parameters."""
        parameters = []
        args = func_node.args
        
        # Regular arguments
        for arg in args.args:
            param = {
                "name": arg.arg,
                "type_hint": self._annotation_to_string(arg.annotation) if arg.annotation else None,
                "default_value": None,
                "is_optional": False
            }
            parameters.append(param)
        
        # Arguments with defaults
        defaults_offset = len(args.args) - len(args.defaults)
        for i, default in enumerate(args.defaults):
            param_index = defaults_offset + i
            if param_index < len(parameters):
                parameters[param_index]["default_value"] = self._node_to_string(default)
                parameters[param_index]["is_optional"] = True
        
        return parameters
    
    def _extract_return_annotation(self, func_node: ast.FunctionDef) -> Optional[str]:
        """Extract return type annotation."""
        if func_node.returns:
            return self._annotation_to_string(func_node.returns)
        return None
    
    def _extract_classes(self) -> List[Dict[str, Any]]:
        """Extract class information."""
        classes = []
        
        if not self.tree:
            return classes
        
        for node in ast.walk(self.tree):
            if isinstance(node, ast.ClassDef):
                class_info = {
                    "name": node.name,
                    "line_number": node.lineno,
                    "docstring": ast.get_docstring(node),
                    "methods": [],
                    "base_classes": [self._node_to_string(base) for base in node.bases]
                }
                
                # Extract methods
                for item in node.body:
                    if isinstance(item, ast.FunctionDef):
                        method_info = {
                            "name": item.name,
                            "line_number": item.lineno,
                            "docstring": ast.get_docstring(item),
                            "parameters": self._extract_parameters(item),
                            "returns": self._extract_return_annotation(item),
                            "decorators": [self._decorator_to_string(d) for d in item.decorator_list],
                            "is_async": isinstance(item, ast.AsyncFunctionDef)
                        }
                        class_info["methods"].append(method_info)
                
                classes.append(class_info)
        
        return classes
    
    def _extract_imports(self) -> List[Dict[str, Any]]:
        """Extract import information."""
        imports = []
        
        if not self.tree:
            return imports
        
        for node in ast.walk(self.tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append({
                        "module": alias.name,
                        "names": [],
                        "alias": alias.asname,
                        "is_from_import": False,
                        "line_number": node.lineno
                    })
            elif isinstance(node, ast.ImportFrom):
                names = [alias.name for alias in node.names]
                imports.append({
                    "module": node.module or "",
                    "names": names,
                    "alias": None,
                    "is_from_import": True,
                    "line_number": node.lineno
                })
        
        return imports
    
    def _detect_cli_framework(self) -> Optional[Dict[str, Any]]:
        """Detect CLI framework usage."""
        imports = self._extract_imports()
        import_modules = [imp["module"] for imp in imports]
        
        if "typer" in import_modules:
            return {
                "name": "typer",
                "version": None,
                "detected_commands": [],
                "main_callable": None
            }
        elif "click" in import_modules:
            return {
                "name": "click",
                "version": None,
                "detected_commands": [],
                "main_callable": None
            }
        elif "argparse" in import_modules:
            return {
                "name": "argparse",
                "version": None,
                "detected_commands": [],
                "main_callable": None
            }
        
        return None
    
    def _annotation_to_string(self, annotation) -> str:
        """Convert AST annotation to string."""
        return self._node_to_string(annotation)
    
    def _node_to_string(self, node) -> str:
        """Convert AST node to string representation."""
        if isinstance(node, ast.Name):
            return node.id
        elif isinstance(node, ast.Constant):
            return repr(node.value)
        elif isinstance(node, ast.Attribute):
            return f"{self._node_to_string(node.value)}.{node.attr}"
        elif isinstance(node, ast.Subscript):
            return f"{self._node_to_string(node.value)}[{self._node_to_string(node.slice)}]"
        else:
            return ast.unparse(node) if hasattr(ast, 'unparse') else str(node)
    
    def _decorator_to_string(self, decorator) -> str:
        """Convert decorator to string."""
        return self._node_to_string(decorator)


class ImportIntrospector(SafeIntrospector):
    """Enhanced introspection with import capability (requires trust)."""
    
    def introspect(self) -> Dict[str, Any]:
        """Perform enhanced introspection with imports."""
        # First do safe introspection
        metadata = super().introspect()
        
        # Then enhance with import-based analysis
        try:
            self._enhance_with_imports(metadata)
        except Exception as e:
            metadata["errors"].append({
                "error_type": "ImportError",
                "message": f"Import-based analysis failed: {str(e)}",
                "line_number": None
            })
        
        return metadata
    
    def _enhance_with_imports(self, metadata: Dict[str, Any]):
        """Enhance metadata using actual imports."""
        # This is where we could actually import the module
        # and inspect it dynamically, but for safety we'll
        # keep it minimal for now
        pass


def calculate_script_hash(script_path: Path) -> str:
    """Calculate hash of script content."""
    content = script_path.read_bytes()
    return hashlib.sha256(content).hexdigest()


def get_python_version() -> str:
    """Get current Python version."""
    return f"Python {sys.version}"


def main():
    parser = argparse.ArgumentParser(description="Pyst Script Introspector")
    parser.add_argument("script_path", type=Path, help="Path to script to introspect")
    parser.add_argument("--mode", choices=["safe", "import"], default="safe", 
                       help="Introspection mode")
    parser.add_argument("--output", type=Path, help="Output file (default: stdout)")
    
    args = parser.parse_args()
    
    if not args.script_path.exists():
        print(f"Error: Script not found: {args.script_path}", file=sys.stderr)
        sys.exit(1)
    
    # Choose introspector based on mode
    if args.mode == "import":
        introspector = ImportIntrospector(args.script_path)
    else:
        introspector = SafeIntrospector(args.script_path)
    
    # Perform introspection
    metadata = introspector.introspect()
    
    # Create result
    result = {
        "schema_version": SCHEMA_VERSION,
        "python_version": get_python_version(),
        "script_hash": calculate_script_hash(args.script_path),
        "metadata": metadata
    }
    
    # Output result
    output_json = json.dumps(result, indent=2, ensure_ascii=False)
    
    if args.output:
        args.output.write_text(output_json, encoding='utf-8')
    else:
        print(output_json)


if __name__ == "__main__":
    main()