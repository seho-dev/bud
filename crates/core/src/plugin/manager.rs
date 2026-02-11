use config::{load_all_plugins, load_plugin_config, PLUGIN_CONFIG_FILE};
use directories::ProjectDirs;
use shared_types::config::{ConfigData, PluginConfigData};
use shared_types::plugin::PluginError;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub type Result<T> = std::result::Result<T, PluginError>;

/// Manages plugin lifecycle and operations
///
/// Uses `Arc<Config>` to share configuration efficiently across multiple components
/// without cloning the potentially large configuration object.
pub struct PluginManager {
  config: Arc<ConfigData>,
  project_data_path: PathBuf,
  plugin_cache: HashMap<String, PluginConfigData>,
}

impl PluginManager {
  /// Gets the project data directory path
  ///
  /// # Returns
  ///
  /// Returns the absolute path to the project's data directory
  ///
  /// # Errors
  ///
  /// Returns `PluginError::ProjectDirsError` if the project directories cannot be determined
  fn get_project_data_path(name: &str) -> Result<PathBuf> {
    let project_path =
      ProjectDirs::from("com", "bud", name).ok_or(PluginError::ProjectDirsError)?;

    Ok(project_path.data_dir().to_path_buf())
  }

  /// Loads all plugin configurations and populates the cache
  ///
  /// On success, the cache contains all successfully loaded plugins.
  /// Individual plugin failures do not cause the entire method to fail.
  ///
  /// # Returns
  ///
  /// Returns the number of successfully loaded plugins
  ///
  /// # Errors
  ///
  /// - Plugins directory not found: `PluginError::LoadError`
  /// - All plugins failed to load: `PluginError::LoadError`
  pub fn get_all(&mut self) -> Result<usize> {
    let plugins = load_all_plugins(&self.project_data_path)
      .map_err(|e| PluginError::LoadError(e.to_string()))?;

    let count = plugins.len();
    self.plugin_cache = plugins;

    Ok(count)
  }

  /// Gets plugin configuration by name
  ///
  /// Reads from cache first. If cache miss, attempts to load from disk and cache it.
  ///
  /// # Arguments
  ///
  /// * `name` - Plugin name
  ///
  /// # Returns
  ///
  /// Returns a reference to the plugin configuration on success
  ///
  /// # Errors
  ///
  /// - Plugin not found or load failed: `PluginError::LoadError`
  pub fn get(&mut self, name: &str) -> Result<&PluginConfigData> {
    if self.plugin_cache.contains_key(name) {
      return Ok(&self.plugin_cache[name]);
    }

    let plugin_dir = self.project_data_path.join(name);

    let config = load_plugin_config(&plugin_dir)
      .map_err(|e| PluginError::LoadError(format!("Failed to load plugin '{}': {}", name, e)))?;

    if config.name != name {
      return Err(PluginError::LoadError(format!(
        "Plugin directory name '{}' does not match {} name '{}'",
        name,
        PLUGIN_CONFIG_FILE,
        config.name
      )));
    }

    self.plugin_cache.insert(name.to_string(), config);
    Ok(&self.plugin_cache[name])
  }

  /// Creates a new `PluginManager` instance
  ///
  /// # Arguments
  ///
  /// * `config` - Shared configuration wrapped in `Arc` for efficient sharing
  ///
  /// # Returns
  ///
  /// Returns a `Result` containing the new `PluginManager` instance
  ///
  /// # Examples
  ///
  /// ```
  /// use std::sync::Arc;
  /// use config::load_config;
  /// use bud_core::plugin::PluginManager;
  ///
  /// let config = Arc::new(load_config()?);
  /// let manager = PluginManager::new(Arc::clone(&config))?;
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  pub fn new(config: Arc<ConfigData>) -> Result<Self> {
    let project_data_path = Self::get_project_data_path(&config.name)?;
    Ok(Self {
      config,
      project_data_path,
      plugin_cache: HashMap::new(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_config() -> ConfigData {
    ConfigData {
      name: "test-app".to_string(),
      version: "0.1.0".to_string(),
      description: "A test application".to_string(),
    }
  }

  #[test_log::test]
  fn test_plugin_manager_new() {
    let config = Arc::new(create_test_config());
    let manager = PluginManager::new(config);

    assert!(manager.is_ok());
    let manager = manager.unwrap();
    assert_eq!(manager.config.name, "test-app");
    assert_eq!(manager.config.version, "0.1.0");
    assert_eq!(manager.config.description, "A test application");
  }

  #[test_log::test]
  fn test_plugin_manager_get_all() {
    let config = Arc::new(create_test_config());
    let mut manager =
      PluginManager::new(Arc::clone(&config)).expect("Failed to create PluginManager");

    let result = manager.get_all();

    // The plugins directory may not exist in test environment, which is expected
    // Either it succeeds with a count, or fails with LoadError due to missing directory
    match result {
      Ok(_count) => {
        // Successfully loaded plugins (directory exists)
        println!("Successfully loaded plugins, count: {}", _count);
      }
      Err(PluginError::LoadError(_)) => {
        // Expected: plugins directory doesn't exist in test environment
        println!("Plugins directory doesn't exist (expected in test environment)");
      }
      Err(e) => {
        panic!("Unexpected error type: {:?}", e);
      }
    }
  }
}
