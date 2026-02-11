use jsonschema::{Draft, JSONSchema};
use log::{error, info};
use serde_json::Value;
use shared_types::config::ConfigError;
use std::fs;
use std::path::Path;

/// Reads and parses JSON content from a file
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// Returns the parsed JSON Value on success
///
/// # Errors
///
/// - File read failure: `ConfigError::IoError`
/// - JSON parse failure: `ConfigError::ParseError`
pub fn read_and_parse_json<P: AsRef<Path>>(path: P) -> Result<Value, ConfigError> {
  let content = fs::read_to_string(path.as_ref())?;
  let value: Value =
    serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;
  Ok(value)
}

/// Validates JSON data using a precompiled schema
///
/// # Arguments
///
/// * `schema` - Precompiled JSON Schema
/// * `value` - JSON data to validate
///
/// # Returns
///
/// Returns Ok(()) on successful validation
///
/// # Errors
///
/// Returns `ConfigError::ValidationError` on validation failure with detailed error messages
pub fn validate_json(schema: &JSONSchema, value: &Value) -> Result<(), ConfigError> {
  if let Err(errors) = schema.validate(value) {
    let error_messages: Vec<String> = errors
      .map(|e| format!("Path '{}': {}", e.instance_path, e))
      .collect();
    let combined_errors = error_messages.join("\n");
    return Err(ConfigError::ValidationError(combined_errors));
  }
  Ok(())
}

/// Compiles a JSON Schema
///
/// # Arguments
///
/// * `schema_str` - JSON Schema string
///
/// # Returns
///
/// Returns the compiled JSONSchema
///
/// # Panics
///
/// - Panics if the schema format is invalid
/// - Panics if schema compilation fails
pub fn compile_schema(schema_str: &str) -> JSONSchema {
  let schema: Value = serde_json::from_str(schema_str).expect("Schema is invalid");
  JSONSchema::options()
    .with_draft(Draft::Draft7)
    .compile(&schema)
    .expect("Failed to compile schema")
}
