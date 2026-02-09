use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigData {
  pub name: String,
  pub version: String,
  pub description: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PluginConfigData {
  pub name: String,
  pub version: String,
  pub description: String,
  pub author: String,
  pub permissions: Vec<String>,
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
