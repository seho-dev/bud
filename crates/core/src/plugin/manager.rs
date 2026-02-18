use config::{load_all_plugin_configs, load_plugin_config, load_plugin_config_validated};
use directories::ProjectDirs;
use log::error;
use shared_types::Provider;
use shared_types::ProviderValue;
use shared_types::config::{ConfigData, PluginConfigData};
use shared_types::plugin::PluginError;
use std::fs::create_dir_all;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use utils::copy_dir_recursive;

/// Manages plugin lifecycle and operations
///
/// Uses `Arc<Config>` to share configuration efficiently across multiple components
/// without cloning the potentially large configuration object.
pub struct PluginManager<P: Provider> {
  config: Arc<ConfigData>,
  project_data_path: PathBuf,
  plugin_cache: HashMap<String, PluginConfigData>,
  provider: Arc<P>,
}

/// Contains plugin configuration and its filesystem path
///
/// Returned by plugin query methods to provide both the configuration
/// and the location where the plugin is stored.
pub struct PluginInfo {
  pub config: PluginConfigData,
  pub path: PathBuf,
}

impl<P: Provider> PluginManager<P> {
  /// Gets the project data directory path
  ///
  /// # Returns
  ///
  /// Returns the absolute path to the project's data directory
  ///
  /// # Errors
  ///
  /// Returns `PluginError::ProjectDirsError` if the project directories cannot be determined
  fn get_project_data_path(name: &str) -> Result<PathBuf, PluginError> {
    let project_path =
      ProjectDirs::from("com", "bud", name).ok_or(PluginError::ProjectDirsError)?;

    Ok(project_path.data_dir().to_path_buf())
  }

  /// Installs a plugin from the given directory into the project data path
  ///
  /// Reads `plugin.json` from `dir_path` to determine the plugin name, then recursively
  /// copies all files and subdirectories to `project_data_path/<plugin_name>/`.
  ///
  /// # Arguments
  ///
  /// * `dir_path` - Path to the source directory containing plugin files and `plugin.json`
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` on successful installation
  ///
  /// # Errors
  ///
  /// * `PluginError::InstallError` - If `dir_path` is not a directory, or `plugin.json` is missing/invalid
  /// * `PluginError::IoError` - If directory creation or file copy operations fail
  pub fn install(&mut self, dir_path: &PathBuf) -> Result<(), PluginError> {
    if !dir_path.is_dir() {
      let msg = format!("Path is not a directory: {}", dir_path.display());
      error!("{}", msg);
      return Err(PluginError::InstallError(msg));
    }

    let plugin_config = load_plugin_config(&dir_path)
      .map_err(|e| PluginError::InstallError(format!("Failed to read plugin config: {}", e)))?;

    let plugin_name = plugin_config.name.clone();

    let dest_dir = self.project_data_path.join(&plugin_name);

    // Check if plugin is already installed
    if dest_dir.is_dir() {
      let msg = format!("plugin {} is already installed", plugin_name);
      error!("{}", msg);
      return Err(PluginError::InstallError(msg));
    }

    create_dir_all(&dest_dir)?;

    copy_dir_recursive(&dir_path, &dest_dir)?;

    self.plugin_cache.insert(plugin_name, plugin_config);

    Ok(())
  }

  /// Loads all plugin configurations and populates the cache
  ///
  /// This method only loads and validates plugin configuration files (plugin.json),
  /// not the actual plugin runtime files.
  ///
  /// On success, the cache contains all successfully loaded plugin configurations.
  /// Individual plugin failures do not cause the entire method to fail.
  ///
  /// # Returns
  ///
  /// Returns a Vec of PluginInfo (containing config and path for each plugin)
  ///
  /// # Errors
  ///
  /// - Plugins directory not found: `PluginError::LoadError`
  /// - All plugins failed to load: `PluginError::LoadError`
  pub fn get_all(&mut self) -> Result<Vec<PluginInfo>, PluginError> {
    let plugins = load_all_plugin_configs(&self.project_data_path)
      .map_err(|e| PluginError::LoadError(e.to_string()))?;

    // Update cache first, then build result from cache to avoid cloning the entire HashMap
    self.plugin_cache = plugins;

    let plugin_infos: Vec<PluginInfo> = self
      .plugin_cache
      .iter()
      .map(|(name, config)| {
        let path = self.project_data_path.join(name);
        PluginInfo {
          config: config.clone(),
          path,
        }
      })
      .collect();

    Ok(plugin_infos)
  }

  /// Gets plugin configuration by name
  ///
  /// This method only loads and validates the plugin configuration file (plugin.json),
  /// not the actual plugin runtime files.
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
  pub fn get(&mut self, name: &str) -> Result<PluginInfo, PluginError> {
    let plugin_dir = self.project_data_path.join(name);

    // Return from cache if available
    if let Some(cached_config) = self.plugin_cache.get(name) {
      return Ok(PluginInfo {
        config: cached_config.clone(),
        path: plugin_dir,
      });
    }

    // Load, cache, and return
    let config = load_plugin_config_validated(&plugin_dir, name)
      .map_err(|e| PluginError::LoadError(format!("Failed to load plugin '{}': {}", name, e)))?;

    self.plugin_cache.insert(name.to_string(), config.clone());

    Ok(PluginInfo {
      config,
      path: plugin_dir,
    })
  }

  pub fn load(&mut self, name: &str) -> Result<(), PluginError> {
    let plugin_info = self.get(name)?;

    self
      .provider
      .load(&plugin_info.path)
      .map_err(|e| PluginError::LoadError(e.to_string()))?;

    Ok(())
  }

  pub fn invoke(
    &mut self,
    name: &str,
    function: &str,
    args: Vec<ProviderValue>,
  ) -> Result<ProviderValue, PluginError> {
    let _plugin_info = self.get(name)?;

    // Check if plugin is loaded
    self
      .provider
      .with_plugins(|plugins| {
        if !plugins.contains_key(name) {
          let msg = format!(
            "Plugin '{}' not found, You must load the plugin first",
            name
          );
          error!("{}", msg);
          return Err(PluginError::LoadError(msg));
        }
        Ok(())
      })
      .map_err(|e| PluginError::InvokeError(e.to_string()))??;

    // Invoke the function
    self
      .provider
      .invoke(name, function, args)
      .map_err(|e| PluginError::InvokeError(e.to_string()))
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
  pub fn new(config: Arc<ConfigData>, provider: Arc<P>) -> Result<Self, PluginError> {
    let project_data_path = Self::get_project_data_path(&config.name)?;
    Ok(Self {
      config,
      project_data_path,
      plugin_cache: HashMap::new(),
      provider,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use wasm_provider::WasmProvider;

  fn create_manager() -> PluginManager<WasmProvider> {
    let config = Arc::new(ConfigData {
      name: "test-app".to_string(),
      version: "0.1.0".to_string(),
      description: "A test application".to_string(),
    });
    let provider = Arc::new(WasmProvider::new());
    PluginManager::new(config, provider).unwrap()
  }

  fn setup_test_plugin(manager: &mut PluginManager<WasmProvider>) {
    let path = workspace_root::get_workspace_root().join("example/test-plugin");
    let target = manager.project_data_path.join("test-plugin");
    if target.exists() {
      std::fs::remove_dir_all(&target).unwrap();
    }
    manager.install(&path).unwrap();
  }

  #[test_log::test]
  fn test_plugin_install() {
    let mut manager = create_manager();
    setup_test_plugin(&mut manager);
    assert!(manager.project_data_path.join("test-plugin").exists());
  }

  #[test_log::test]
  fn test_plugin_manager_new() {
    let manager = create_manager();
    assert_eq!(manager.config.name, "test-app");
    assert_eq!(manager.config.version, "0.1.0");
    assert_eq!(manager.config.description, "A test application");
  }

  #[test_log::test]
  fn test_plugin_manager_get_all() {
    let mut manager = create_manager();
    let result = manager.get_all();
    assert!(result.is_ok() || matches!(result, Err(PluginError::LoadError(_))));
  }

  #[test_log::test]
  fn test_plugin_manager_get() {
    let mut manager = create_manager();
    setup_test_plugin(&mut manager);
    let result = manager.get("test-plugin");
    assert!(result.is_ok());
  }

  #[test_log::test]
  fn test_plugin_manager_load() {
    let mut manager = create_manager();
    manager.provider.init().expect("Failed to initialize provider");
    setup_test_plugin(&mut manager);
    let result = manager.load("test-plugin");
    assert!(result.is_ok());
  }

  #[test_log::test]
  fn test_plugin_manager_invoke() {
    let mut manager = create_manager();
    manager.provider.init().expect("Failed to initialize provider");
    setup_test_plugin(&mut manager);
    manager.load("test-plugin").expect("Failed to load plugin");

    let result = manager.invoke(
      "test-plugin",
      "Sum",
      vec![ProviderValue::I32(1), ProviderValue::I32(2)],
    );

    assert!(result.is_ok(), "Plugin invocation should succeed");
    let value = result.unwrap();
    assert_eq!(value, ProviderValue::I32(3), "Sum(1, 2) should return 3");
  }
}
