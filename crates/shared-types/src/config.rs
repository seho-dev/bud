use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigData {
  pub name: String,
  pub version: String,
  pub description: String,
}

// Permission<T> is a generic enum representing a permission dimension that can be either:
//   - a boolean shorthand (true = fully allow, false = fully deny)
//   - a detailed config object of type T
//
// IMPORTANT: In serde's untagged mode, variants are tried in declaration order.
// `Bool` must come before `Config`, otherwise JSON `true` would be attempted as an object and fail.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Permission<T> {
  Bool(bool),
  Config(T),
}

/// Detailed configuration for stdio permissions, controlling access to stdin/stdout/stderr.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct StdioPermission {
  pub stdin: Option<bool>,
  pub stdout: Option<bool>,
  pub stderr: Option<bool>,
}

/// Detailed configuration for filesystem permissions, specifying allowed read and write paths.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct FilesystemPermission {
  pub read: Option<Vec<String>>,
  pub write: Option<Vec<String>>,
}

/// Detailed configuration for network permissions, specifying the list of allowed hosts.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct NetworkPermission {
  pub allowed_hosts: Option<Vec<String>>,
}

/// Detailed configuration for environment variable permissions, controlling inheritance and allowed keys.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct EnvPermission {
  pub inherit: Option<bool>,
  pub keys: Option<Vec<String>>,
}

/// Detailed configuration for process-level permissions (e.g. whether the plugin may call exit).
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct ProcessPermission {
  pub exit: Option<bool>,
}

/// Aggregated plugin permissions. Each field represents one permission dimension.
/// A field may be Bool (quick toggle), a Config object (fine-grained), or absent (defaults to deny).
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct Permissions {
  pub stdio: Option<Permission<StdioPermission>>,
  pub filesystem: Option<Permission<FilesystemPermission>>,
  pub network: Option<Permission<NetworkPermission>>,
  pub env: Option<Permission<EnvPermission>>,
  pub process: Option<Permission<ProcessPermission>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PluginConfigData {
  pub name: String,
  pub version: String,
  pub description: String,
  pub author: String,
  pub permissions: Option<Permissions>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
  #[error(
    "Configuration file not found: {0}\n\nPlease ensure config file exists in the project root directory."
  )]
  FileNotFound(String),

  #[error("Configuration parsing error: {0}")]
  ParseError(String),

  #[error("Configuration validation failed:\n{0}")]
  ValidationError(String),

  #[error("File reading error: {0}")]
  IoError(#[from] std::io::Error),
}
