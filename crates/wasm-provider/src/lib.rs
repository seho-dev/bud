use log::{error, info};
use shared_types::{Provider, ProviderError, ProviderValue};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use wasmtime::{Config, Engine, Module};

#[derive(Clone)]
pub struct WasmInstance {
  pub engine: Arc<Engine>,
  pub modules: Arc<RwLock<HashMap<String, Module>>>,
}

impl WasmInstance {
  pub fn new(engine: Engine) -> Self {
    Self {
      engine: Arc::new(engine),
      modules: Arc::new(RwLock::new(HashMap::new())),
    }
  }
}

pub struct WasmProvider {
  instance: Arc<RwLock<Option<WasmInstance>>>,
}

impl WasmProvider {
  #[must_use]
  pub fn new() -> Self {
    Self {
      instance: Arc::new(RwLock::new(None)),
    }
  }
}

impl Provider for WasmProvider {
  type Instance = WasmInstance;
  const MAIN_FILE: &'static str = "main.wasm";

  fn init(&self) -> Result<Self::Instance, ProviderError> {
    info!("Initializing WasmProvider");

    let config = Config::default();
    let engine = Engine::new(&config).map_err(|_| ProviderError::InitFailed)?;

    let new_instance = WasmInstance::new(engine);

    let mut instance_guard = self
      .instance
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    // Store the instance for use by other Provider methods (like load)
    *instance_guard = Some(new_instance.clone());

    info!("WasmProvider initialized successfully");
    Ok(new_instance)
  }

  fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), ProviderError> {
    let plugin_dir = path.as_ref();
    let wasm_file = plugin_dir.join(Self::MAIN_FILE);

    if !wasm_file.is_file() {
      let msg = format!("{} not found: {}", Self::MAIN_FILE, wasm_file.display());
      error!("{}", msg);
      return Err(ProviderError::LoadFailed(msg));
    }

    let plugin_name = plugin_dir
      .file_name()
      .and_then(|n| n.to_str())
      .ok_or_else(|| {
        ProviderError::LoadFailed(format!("Invalid plugin path: {}", plugin_dir.display()))
      })?
      .to_string();

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

    let module = Module::from_file(&instance.engine, &wasm_file).map_err(|e| {
      ProviderError::LoadFailed(format!("Failed to compile plugin '{}': {}", plugin_name, e))
    })?;

    let mut modules = instance
      .modules
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if modules.contains_key(&plugin_name) {
      info!(
        "Plugin '{}' already loaded, using cached module",
        plugin_name
      );
      return Ok(());
    }

    modules.insert(plugin_name.clone(), module);

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
