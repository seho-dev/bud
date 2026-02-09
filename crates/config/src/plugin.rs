use jsonschema::JSONSchema;
use log::{error, warn};
use once_cell::sync::Lazy;
use shared_types::config::{ConfigError, PluginConfigData};
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::Path;

use crate::common::{compile_schema, read_and_parse_json, validate_json};

const PLUGIN_CONFIG_FILE: &str = "plugin.json";

static PLUGIN_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9_-]+$"
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+(-[a-zA-Z0-9.-]+)?(\\+[a-zA-Z0-9.-]+)?$"
    },
    "description": {
      "type": "string",
      "minLength": 1
    },
    "author": {
      "type": "string",
      "minLength": 1
    },
    "permissions": {
      "type": "array",
      "items": { "type": "string" },
      "uniqueItems": true
    }
  },
  "required": ["name", "version", "description", "author", "permissions"]
}"#;

static COMPILED_PLUGIN_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| compile_schema(PLUGIN_SCHEMA));

/// Loads plugin configuration from the specified plugin directory
///
/// # Arguments
///
/// * `plugin_dir` - Path to the plugin directory
///
/// # Returns
///
/// Returns the parsed plugin configuration on success
///
/// # Errors
///
/// - plugin.json not found: `ConfigError::FileNotFound`
/// - JSON parse failure: `ConfigError::ParseError`
/// - Schema validation failure: `ConfigError::ValidationError`
pub fn load_plugin_config<P: AsRef<Path>>(plugin_dir: P) -> Result<PluginConfigData, ConfigError> {
  let config_path = plugin_dir.as_ref().join(PLUGIN_CONFIG_FILE);

  if !config_path.exists() {
    return Err(ConfigError::FileNotFound(config_path.display().to_string()));
  }

  parse_plugin_config(&config_path)
}

fn parse_plugin_config<P: AsRef<Path>>(path: P) -> Result<PluginConfigData, ConfigError> {
  let value = read_and_parse_json(&path)?;
  validate_json(&COMPILED_PLUGIN_SCHEMA, &value)?;

  let config: PluginConfigData =
    serde_json::from_value(value).map_err(|e| ConfigError::ParseError(e.to_string()))?;

  Ok(config)
}

/// Loads all plugin configurations from the plugins directory
///
/// # Arguments
///
/// * `plugins_dir` - Path to the plugins root directory
///
/// # Returns
///
/// Returns HashMap<plugin_name, plugin_config> on success
///
/// # Errors
///
/// - Plugins directory not found: `ConfigError::FileNotFound`
/// - All plugins failed to load: `ConfigError::ValidationError`
///
/// # Fault Tolerance
///
/// Individual plugin failures are logged (ERROR level) and skipped without interrupting
/// the overall loading process. Only returns an error if all plugins fail to load.
pub fn load_all_plugins<P: AsRef<Path>>(
  plugins_dir: P,
) -> Result<HashMap<String, PluginConfigData>, ConfigError> {
  let plugins_dir = plugins_dir.as_ref();

  if !plugins_dir.exists() {
    return Err(ConfigError::FileNotFound(format!(
      "Plugins directory not found: {}",
      plugins_dir.display()
    )));
  }

  let mut plugins = HashMap::new();
  let mut errors = Vec::new();

  let entries = read_dir(plugins_dir)?;

  for entry in entries {
    let entry = match entry {
      Ok(e) => e,
      Err(e) => {
        warn!("Failed to read directory entry: {}", e);
        continue;
      }
    };

    let path = entry.path();

    if !path.is_dir() {
      continue;
    }

    let plugin_name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("unknown");

    match load_plugin_config(&path) {
      Ok(config) => {
        let config_name = config.name.clone();

        if config_name != plugin_name {
          warn!(
            "Plugin directory name '{}' does not match plugin.json name '{}'",
            plugin_name, config_name
          );
        }

        if plugins.insert(config_name.clone(), config).is_some() {
          error!("Duplicate plugin name detected: {}", config_name);
        }
      }
      Err(e) => {
        error!("Failed to load plugin from '{}': {}", plugin_name, e);
        errors.push(format!("{}: {}", plugin_name, e));
      }
    }
  }

  if plugins.is_empty() && !errors.is_empty() {
    return Err(ConfigError::ValidationError(format!(
      "Failed to load any plugins:\n{}",
      errors.join("\n")
    )));
  }

  Ok(plugins)
}
