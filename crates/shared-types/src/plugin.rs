use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
  #[error("Failed to initialize plugin manager: {0}")]
  InitError(String),

  #[error("Failed to load plugin: {0}")]
  LoadError(String),

  #[error("Failed to determine project directories")]
  ProjectDirsError,

  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
}

pub struct Plugin {
  pub name: String,
  pub version: String,
  pub description: String,
  pub path: PathBuf,
}
