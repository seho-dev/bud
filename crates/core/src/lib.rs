pub mod plugin;

use config::load_config;
use log::{debug, info};
use plugin::PluginManager;
use shared_types::config::{ConfigData, ConfigError};
use shared_types::plugin::PluginError;
use shared_types::{Provider, ProviderError};
use std::sync::{Arc, Once};

static INIT: Once = Once::new();

pub fn init_logger() {
  INIT.call_once(|| {
    env_logger::Builder::from_default_env()
      .filter_level(log::LevelFilter::Info)
      .format_timestamp_secs()
      .format_module_path(true)
      .format_target(false)
      .init();

    info!("The initialization of the log system is complete");
    debug!("Debug level log is enabled");
  });
}

#[derive(thiserror::Error, Debug)]
pub enum BudCoreError {
  #[error(transparent)]
  Config(#[from] ConfigError),

  #[error(transparent)]
  Plugin(#[from] PluginError),

  #[error("Provider initialization failed: {0}")]
  ProviderInitFailed(ProviderError),
}

/// BudCore builder.
///
/// Uses generic parameter `P` to represent the concrete Provider type.
/// Enables zero-cost static dispatch.
///
/// # Type Parameters
///
/// * `P` - Concrete type implementing the `Provider` trait (e.g., `WasmProvider`)
///
/// # Examples
///
/// ```
/// use core::BudCore;
/// use wasm_provider::WasmProvider;
///
/// let provider = WasmProvider::new();
/// let core = BudCore::builder(provider).build()?;
/// ```
pub struct BudCoreBuilder<P: Provider> {
  provider: Arc<P>,
}

impl<P: Provider> BudCoreBuilder<P> {
  /// Create a new BudCore builder.
  ///
  /// # Arguments
  ///
  /// * `provider` - Instance implementing the Provider trait
  ///
  /// # Examples
  ///
  /// ```
  /// let builder = BudCoreBuilder::new(WasmProvider::new());
  /// ```
  #[must_use]
  pub fn new(provider: P) -> Self {
    BudCoreBuilder {
      provider: Arc::new(provider),
    }
  }

  /// Build a BudCore instance.
  ///
  /// Steps performed:
  /// 1. Initialize the logging system
  /// 2. Load configuration file
  /// 3. Initialize the Provider runtime instance
  /// 4. Initialize the plugin manager
  ///
  /// # Errors
  ///
  /// - `BudCoreError::Config` - Configuration loading failed
  /// - `BudCoreError::ProviderInitFailed` - Provider initialization failed
  /// - `BudCoreError::Plugin` - Plugin manager initialization failed
  pub fn build(self) -> Result<BudCore<P>, BudCoreError> {
    init_logger();
    info!("BudCore Start Init");

    let config = Arc::new(load_config()?);
    info!("Config: {:?}", config);

    let provider_instance = self
      .provider
      .init()
      .map_err(BudCoreError::ProviderInitFailed)?;

    let plugin_manager = PluginManager::new(Arc::clone(&config), Arc::clone(&self.provider))?;

    Ok(BudCore {
      package_name: config.name.clone(),
      config,
      provider: self.provider,
      provider_instance,
      plugin_manager,
    })
  }
}

/// BudCore instance.
///
/// Contains core application components: config, Provider runtime, and plugin manager.
///
/// # Type Parameters
///
/// * `P` - Concrete type implementing the `Provider` trait (e.g., `WasmProvider`)
///
/// # Examples
///
/// ```
/// use core::BudCore;
/// use wasm_provider::WasmProvider;
///
/// let core: BudCore<WasmProvider> = BudCore::builder(WasmProvider::new())
///     .build()?;
///
/// println!("Package: {}", core.package_name);
/// println!("Main file: {}", WasmProvider::MAIN_FILE);
/// ```
pub struct BudCore<P: Provider> {
  pub package_name: String,
  pub config: Arc<ConfigData>,
  provider: Arc<P>,
  provider_instance: P::Instance,
  plugin_manager: PluginManager<P>,
}

impl<P: Provider> BudCore<P> {
  /// Create a BudCore builder (recommended construction method).
  ///
  /// # Arguments
  ///
  /// * `provider` - Instance implementing the Provider trait
  ///
  /// # Examples
  ///
  /// ```
  /// use core::BudCore;
  /// use wasm_provider::WasmProvider;
  ///
  /// let core = BudCore::builder(WasmProvider::new()).build()?;
  /// ```
  pub fn builder(provider: P) -> BudCoreBuilder<P> {
    BudCoreBuilder::new(provider)
  }

  /// Get a mutable reference to the Provider instance.
  ///
  /// Used when calling Provider runtime methods.
  ///
  /// # Examples
  ///
  /// ```
  /// let mut core = BudCore::builder(WasmProvider::new()).build()?;
  /// let instance = core.provider_instance_mut();
  /// ```
  pub fn provider_instance_mut(&mut self) -> &mut P::Instance {
    &mut self.provider_instance
  }

  /// Get a reference to the Provider.
  ///
  /// # Examples
  ///
  /// ```
  /// let core = BudCore::builder(WasmProvider::new()).build()?;
  /// let provider = core.provider();
  /// ```
  pub fn provider(&self) -> &P {
    &self.provider
  }

  /// Get a mutable reference to the PluginManager.
  ///
  /// Used for plugin operations like loading, getting plugin info, etc.
  ///
  /// # Examples
  ///
  /// ```
  /// let mut core = BudCore::builder(WasmProvider::new()).build()?;
  /// let manager = core.plugin_manager_mut();
  /// let plugin_info = manager.get("my-plugin")?;
  /// ```
  pub fn plugin_manager_mut(&mut self) -> &mut plugin::PluginManager<P> {
    &mut self.plugin_manager
  }
}
