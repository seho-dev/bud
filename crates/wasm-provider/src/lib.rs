use log::{error, info};
use shared_types::{Provider, ProviderError, ProviderValue};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use utils::provider_json::{args_to_json, json_to_provider_value};
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    world: "bud-plugin",
    path: "../../wit/bud.wit",
});

use bud::sdk::host::{Host, LogLevel};

pub struct PluginState {
  wasi: WasiCtx,
  table: ResourceTable,
}

impl WasiView for PluginState {
  fn ctx(&mut self) -> WasiCtxView<'_> {
    WasiCtxView {
      ctx: &mut self.wasi,
      table: &mut self.table,
    }
  }
}

// Implements the host-side of the WIT `interface host`, called by plugins at runtime.
impl Host for PluginState {
  fn log(&mut self, level: LogLevel, msg: String) {
    match level {
      LogLevel::Debug => log::debug!("[plugin] {}", msg),
      LogLevel::Info => log::info!("[plugin] {}", msg),
      LogLevel::Warn => log::warn!("[plugin] {}", msg),
      LogLevel::Error => log::error!("[plugin] {}", msg),
    }
  }

  fn emit(&mut self, event: String, data: String) {
    println!("emit: {}", event);
    println!("data: {}", data);
  }

  fn get_config(&mut self, _key: String) -> Option<String> {
    None
  }
}

// Holds the wasmtime Store and the generated bindings for one loaded plugin component.
pub struct PluginInstance {
  pub store: Store<PluginState>,
  pub bindings: BudPlugin,
  pub module_name: String,
}

// Shared engine and linker; cloned cheaply via Arc for each plugin load.
#[derive(Clone)]
pub struct WasmInstance {
  pub engine: Arc<Engine>,
  pub linker: Arc<Linker<PluginState>>,
}

pub struct WasmProvider {
  instance: Arc<RwLock<Option<WasmInstance>>>,
  pub plugins: Arc<Mutex<HashMap<String, PluginInstance>>>,
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
  type PluginInstance = PluginInstance;
  const MAIN_FILE: &'static str = "main.wasm";

  fn with_plugins<F, R>(&self, f: F) -> Result<R, ProviderError>
  where
    F: FnOnce(&HashMap<String, PluginInstance>) -> R,
  {
    let plugins = self.plugins.lock().unwrap_or_else(|p| p.into_inner());
    Ok(f(&plugins))
  }

  fn init(&self) -> Result<Self::Instance, ProviderError> {
    info!("Initializing WasmProvider (Component Model)");

    let mut config = Config::default();
    config.wasm_component_model(true);

    let engine = Engine::new(&config).map_err(|e| {
      error!("Failed to create Engine: {}", e);
      ProviderError::InitFailed
    })?;

    let mut linker: Linker<PluginState> = Linker::new(&engine);

    // Register WASI preview2 host functions (stdio, clocks, etc.)
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(|e| {
      error!("Failed to add WASI to linker: {}", e);
      ProviderError::InitFailed
    })?;

    // Register bud host functions defined in wit/bud.wit `interface host`
    BudPlugin::add_to_linker::<PluginState, HasSelf<PluginState>>(&mut linker, |state| state)
      .map_err(|e| {
        error!("Failed to add host bindings to linker: {}", e);
        ProviderError::InitFailed
      })?;

    let new_instance = WasmInstance {
      engine: Arc::new(engine),
      linker: Arc::new(linker),
    };

    *self.instance.write().unwrap_or_else(|p| p.into_inner()) = Some(new_instance.clone());

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

    let instance_guard = self.instance.read().unwrap_or_else(|p| p.into_inner());
    let instance = instance_guard.as_ref().ok_or_else(|| {
      ProviderError::LoadFailed("Provider not initialized. Call init() first.".to_string())
    })?;

    info!(
      "Compiling component '{}' from {}",
      plugin_name,
      wasm_file.display()
    );

    let component = Component::from_file(&instance.engine, &wasm_file).map_err(|e| {
      error!("Failed to compile component '{}': {}", plugin_name, e);
      ProviderError::LoadFailed(format!(
        "Failed to compile component '{}': {}",
        plugin_name, e
      ))
    })?;

    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(
      &instance.engine,
      PluginState {
        wasi,
        table: ResourceTable::new(),
      },
    );

    // Instantiate the component and wire up host↔plugin bindings
    let bindings =
      BudPlugin::instantiate(&mut store, &component, &instance.linker).map_err(|e| {
        ProviderError::LoadFailed(format!("Failed to instantiate '{}': {}", plugin_name, e))
      })?;

    bindings
      .bud_sdk_plugin()
      .call_on_load(&mut store)
      .map_err(|e| ProviderError::LoadFailed(format!("on-load trap: {}", e)))?
      .map_err(|e| ProviderError::LoadFailed(format!("on-load error: {}", e)))?;

    self
      .plugins
      .lock()
      .unwrap_or_else(|p| p.into_inner())
      .insert(
        plugin_name.clone(),
        PluginInstance {
          store,
          bindings,
          module_name: plugin_name.clone(),
        },
      );

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
    Ok(())
  }

  fn invoke(
    &self,
    plugin_name: &str,
    function: &str,
    args: Vec<ProviderValue>,
  ) -> Result<ProviderValue, ProviderError> {
    let mut plugins = self.plugins.lock().unwrap_or_else(|p| p.into_inner());

    let plugin = plugins
      .get_mut(plugin_name)
      .ok_or_else(|| ProviderError::LoadFailed(format!("Plugin '{}' not found", plugin_name)))?;

    // WIT only supports string args; serialize ProviderValue array to JSON
    let args_json = serde_json::to_string(&args_to_json(&args))
      .map_err(|e| ProviderError::InvocationFailed(e.to_string()))?;

    let result_json = plugin
      .bindings
      .bud_sdk_plugin()
      .call_on_invoke(&mut plugin.store, function, &args_json)
      .map_err(|e| ProviderError::InvocationFailed(e.to_string()))?
      .map_err(ProviderError::InvocationFailed)?;

    let value = serde_json::from_str(&result_json)
      .map_err(|e| ProviderError::InvocationFailed(e.to_string()))?;

    Ok(json_to_provider_value(&value))
  }

  fn unload(&self, _instance: Self::Instance) -> Result<(), ProviderError> {
    Ok(())
  }
}
