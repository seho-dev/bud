use log::{error, info};
use shared_types::{Provider, ProviderError, ProviderValue};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{self, WasiP1Ctx};

/// WASI state for each plugin instance.
///
/// Contains the WASI context providing access to filesystem, environment, and stdio.
pub struct PluginState {
  /// WASI Preview 1 context
  pub wasi: WasiP1Ctx,
}

/// Plugin instance with isolated Store and instantiated Module.
///
/// Each plugin has its own Store, ensuring complete memory and state isolation
/// between different plugins.
pub struct PluginInstance {
  /// Isolated Store for this plugin with WASI state
  pub store: Store<PluginState>,
  /// Instantiated WebAssembly module
  pub instance: Instance,
  /// Plugin name
  pub module_name: String,
}

/// WasmProvider's runtime instance.
///
/// Contains shared Engine and Linker that can be used across multiple plugins.
#[derive(Clone)]
pub struct WasmInstance {
  /// Shared WebAssembly Engine
  pub engine: Arc<Engine>,
  /// Shared Linker for host function injection with WASI support
  pub linker: Arc<Linker<PluginState>>,
}

/// WASM Provider implementation using wasmtime.
pub struct WasmProvider {
  /// Shared WasmInstance
  instance: Arc<RwLock<Option<WasmInstance>>>,
  /// Plugin instances with isolated Stores (Mutex allows Send but not Sync)
  plugins: Arc<Mutex<HashMap<String, PluginInstance>>>,
}

impl WasmProvider {
  #[must_use]
  pub fn new() -> Self {
    Self {
      instance: Arc::new(RwLock::new(None)),
      plugins: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

impl Provider for WasmProvider {
  type Instance = WasmInstance;
  const MAIN_FILE: &'static str = "main.wasm";

  fn init(&self) -> Result<Self::Instance, ProviderError> {
    info!("Initializing WasmProvider");

    // Create Engine with default configuration
    let config = Config::default();
    let engine = Engine::new(&config).map_err(|e| {
      error!("Failed to create Engine: {}", e);
      ProviderError::InitFailed
    })?;

    // Create Linker with WASI support
    let mut linker = Linker::new(&engine);

    // Add WASI Preview 1 functions to the linker
    p1::add_to_linker_sync(&mut linker, |state: &mut PluginState| &mut state.wasi).map_err(
      |e| {
        error!("Failed to add WASI to linker: {}", e);
        ProviderError::InitFailed
      },
    )?;

    let new_instance = WasmInstance {
      engine: Arc::new(engine),
      linker: Arc::new(linker),
    };

    // Store the instance for use by other Provider methods (like load)
    let mut instance_guard = self
      .instance
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    *instance_guard = Some(new_instance.clone());

    info!("WasmProvider initialized successfully with WASI support");
    Ok(new_instance)
  }

  fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), ProviderError> {
    let plugin_dir = path.as_ref();
    let wasm_file = plugin_dir.join(Self::MAIN_FILE);

    // Validate plugin file exists
    if !wasm_file.is_file() {
      let msg = format!("{} not found: {}", Self::MAIN_FILE, wasm_file.display());
      error!("{}", msg);
      return Err(ProviderError::LoadFailed(msg));
    }

    // Extract plugin name from directory
    let plugin_name = plugin_dir
      .file_name()
      .and_then(|n| n.to_str())
      .ok_or_else(|| {
        ProviderError::LoadFailed(format!("Invalid plugin path: {}", plugin_dir.display()))
      })?
      .to_string();

    // Get the WasmInstance
    let instance_guard = self
      .instance
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    let instance = instance_guard.as_ref().ok_or_else(|| {
      ProviderError::LoadFailed("Provider not initialized. Call init() first.".to_string())
    })?;

    info!(
      "Compiling plugin '{}' from {}",
      plugin_name,
      wasm_file.display()
    );

    // Compile the WASM module
    let module = Module::from_file(&instance.engine, &wasm_file).map_err(|e| {
      error!("Failed to compile plugin '{}': {}", plugin_name, e);
      ProviderError::LoadFailed(format!("Failed to compile plugin '{}': {}", plugin_name, e))
    })?;

    // Create WASI context for this plugin
    let wasi = WasiCtxBuilder::new()
      .inherit_stdio() // Inherit stdin/stdout/stderr from host
      .inherit_args() // Inherit command-line arguments
      .inherit_env() // Inherit environment variables
      .build_p1();

    let plugin_state = PluginState { wasi };

    // Create isolated Store for this plugin with WASI state
    let mut store = Store::new(&instance.engine, plugin_state);

    // Instantiate the module using Linker
    let wasm_instance = instance
      .linker
      .instantiate(&mut store, &module)
      .map_err(|e| {
        error!("Failed to instantiate plugin '{}': {}", plugin_name, e);
        ProviderError::LoadFailed(format!(
          "Failed to instantiate plugin '{}': {}",
          plugin_name, e
        ))
      })?;

    info!("Plugin '{}' instantiated successfully", plugin_name);

    // Create PluginInstance with isolated Store
    let plugin_instance = PluginInstance {
      store,
      instance: wasm_instance,
      module_name: plugin_name.clone(),
    };

    // Store the PluginInstance
    let mut plugins = self
      .plugins
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if plugins.contains_key(&plugin_name) {
      info!(
        "Plugin '{}' already loaded, replacing with new instance",
        plugin_name
      );
    }

    plugins.insert(plugin_name.clone(), plugin_instance);

    info!("Plugin '{}' loaded successfully", plugin_name);

    Ok(())
  }

  fn inject(
    &self,
    _instance: &mut Self::Instance,
    _functions: &[(
      &str,
      &dyn Fn(Vec<ProviderValue>) -> Result<ProviderValue, ProviderError>,
    )],
  ) -> Result<(), ProviderError> {
    // Implementation for injecting functions into the WASM store
    todo!()
  }

  fn invoke(
    &self,
    _instance: &mut Self::Instance,
    _function: &str,
    _args: Vec<ProviderValue>,
  ) -> Result<ProviderValue, ProviderError> {
    // Implementation for invoking a function in the WASM store
    todo!()
  }

  fn unload(&self, _instance: Self::Instance) -> Result<(), ProviderError> {
    // Implementation for unloading the WASM store
    todo!()
  }
}
