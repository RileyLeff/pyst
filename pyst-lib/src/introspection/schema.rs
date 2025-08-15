use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResult {
    pub schema_version: String,
    pub python_version: String,
    pub script_hash: String,
    pub metadata: ScriptMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub docstring: Option<String>,
    pub pep723_metadata: Option<Pep723Metadata>,
    pub dependencies: Vec<Dependency>,
    pub entry_points: Vec<EntryPoint>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
    pub cli_framework: Option<CliFramework>,
    pub errors: Vec<IntrospectionError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pep723Metadata {
    pub dependencies: Vec<String>,
    pub requires_python: Option<String>,
    pub tool_config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_spec: Option<String>,
    pub source: DependencySource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    Pep723,
    Import,
    RequirementsTxt,
    Pyproject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub name: String,
    pub callable: String,
    pub module: Option<String>,
    pub entry_type: EntryPointType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryPointType {
    Function,
    Class,
    CliCommand,
    Main,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line_number: u32,
    pub docstring: Option<String>,
    pub parameters: Vec<ParameterInfo>,
    pub returns: Option<String>,
    pub decorators: Vec<String>,
    pub is_async: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_hint: Option<String>,
    pub default_value: Option<String>,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub line_number: u32,
    pub docstring: Option<String>,
    pub methods: Vec<FunctionInfo>,
    pub base_classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module: String,
    pub names: Vec<String>,
    pub alias: Option<String>,
    pub is_from_import: bool,
    pub line_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliFramework {
    pub name: String,
    pub version: Option<String>,
    pub detected_commands: Vec<String>,
    pub main_callable: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionError {
    pub error_type: ErrorType,
    pub message: String,
    pub line_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    SyntaxError,
    ImportError,
    TypeError,
    RuntimeError,
    PermissionDenied,
}

impl Default for ScriptMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            description: None,
            docstring: None,
            pep723_metadata: None,
            dependencies: Vec::new(),
            entry_points: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            imports: Vec::new(),
            cli_framework: None,
            errors: Vec::new(),
        }
    }
}