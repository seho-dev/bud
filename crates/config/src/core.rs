use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use shared_types::config::{ConfigData, ConfigError};
use std::env;
use std::path::Path;

use crate::common::{compile_schema, read_and_parse_json, validate_json};

const DEFAULT_CONFIG_FILE: &str = "bud.json";

static CONFIG_SCHEMA: &str = r#"{
    "type": "object",
    "properties": {
        "name": {
            "type": "string",
            "description": "The name of the application."
        },
        "version": {
            "type": "string",
            "description": "The version of the application."
        },
        "description": {
            "type": "string",
            "description": "A short description of the application."
        }
    },
    "required": ["name", "version", "description"]
} "#;

static COMPILED_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| compile_schema(CONFIG_SCHEMA));

pub fn load_config() -> Result<ConfigData, ConfigError> {
  let current_dir = env::current_dir()?;
  let config_path = current_dir.join(DEFAULT_CONFIG_FILE);

  if !config_path.exists() {
    return Err(ConfigError::FileNotFound(config_path.display().to_string()));
  }

  parse_config(&config_path)
}

fn parse_config<P: AsRef<Path>>(path: P) -> Result<ConfigData, ConfigError> {
  let value = read_and_parse_json(&path)?;
  validate_json(&COMPILED_SCHEMA, &value)?;

  let config: ConfigData =
    serde_json::from_value(value).map_err(|e| ConfigError::ParseError(e.to_string()))?;

  Ok(config)
}
