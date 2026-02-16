use log::{error, info};
use shared_types::{Provider, ProviderError, ProviderValue};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, Val};
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

/// Convert `ProviderValue` to `wasmtime::Val`.
///
/// Only supports WASM native numeric types (I32, I64, F32, F64).
/// Other types will return an error with guidance on proper usage.
fn provider_value_to_val(value: &ProviderValue) -> Result<Val, ProviderError> {
  match value {
    // Supported numeric types
    ProviderValue::I32(i) => Ok(Val::I32(*i)),
    ProviderValue::I64(i) => Ok(Val::I64(*i)),
    ProviderValue::F32(f) => Ok(Val::F32(f.to_bits())),
    ProviderValue::F64(f) => Ok(Val::F64(f.to_bits())),

    // Unsupported types with helpful error messages
    ProviderValue::Null => Err(ProviderError::InvocationFailed(
      "Null type not supported for WASM calls. Use I32(0) if a zero value is needed.".to_string(),
    )),
    ProviderValue::Bool(_) => Err(ProviderError::InvocationFailed(
      "Bool type not supported for WASM calls. Convert to I32(0) for false or I32(1) for true."
        .to_string(),
    )),
    ProviderValue::String(_) | ProviderValue::Array(_) | ProviderValue::Object(_) => {
      Err(ProviderError::InvocationFailed(
        "Complex types (String, Array, Object) not supported for WASM calls.".to_string(),
      ))
    }
  }
}

/// Convert `wasmtime::Val` to `ProviderValue`.
///
/// Only supports WASM native numeric types (I32, I64, F32, F64).
/// V128 and Ref types are not supported and will return an error.
fn val_to_provider_value(val: &Val) -> Result<ProviderValue, ProviderError> {
  match val {
    // Supported numeric types
    Val::I32(i) => Ok(ProviderValue::I32(*i)),
    Val::I64(i) => Ok(ProviderValue::I64(*i)),
    Val::F32(bits) => Ok(ProviderValue::F32(f32::from_bits(*bits))),
    Val::F64(bits) => Ok(ProviderValue::F64(f64::from_bits(*bits))),

    // WASM-specific types (not supported)
    Val::V128(_) => Err(ProviderError::InvocationFailed(
      "V128 type not supported. WASM functions should return basic numeric types only.".to_string(),
    )),
    // Catch-all for any future types
    _ => Err(ProviderError::InvocationFailed(format!(
      "Unsupported WASM value type: {:?}",
      val
    ))),
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
    let plugins = self.plugins.lock().unwrap_or_else(
      |poisoned: std::sync::PoisonError<
        std::sync::MutexGuard<'_, HashMap<String, PluginInstance>>,
      >| poisoned.into_inner(),
    );

    Ok(f(&plugins))
  }

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
    _plugin_name: &str,
    _function: &str,
    _args: Vec<ProviderValue>,
  ) -> Result<ProviderValue, ProviderError> {
    let mut plugins = self.plugins.lock().unwrap_or_else(
      |poisoned: std::sync::PoisonError<
        std::sync::MutexGuard<'_, HashMap<String, PluginInstance>>,
      >| poisoned.into_inner(),
    );

    let plugin = plugins.get_mut(_plugin_name).ok_or_else(|| {
      error!("Plugin '{}' not found", _plugin_name);
      ProviderError::LoadFailed(format!("Plugin '{}' not found", _plugin_name))
    })?;

    let func = plugin
      .instance
      .get_func(&mut plugin.store, _function)
      .ok_or_else(|| {
        let error_msg = format!(
          "Function '{}' not found in plugin '{}'",
          _function, _plugin_name
        );
        error!("{}", error_msg);
        ProviderError::InvocationFailed(error_msg)
      })?;

    let wasm_args: Vec<Val> = _args
      .iter()
      .map(provider_value_to_val)
      .collect::<Result<Vec<Val>, ProviderError>>()?;

    // Prepare results buffer based on function signature
    let func_type = func.ty(&plugin.store);

    // Validate that all return types are supported
    for val_type in func_type.results() {
      match val_type {
        wasmtime::ValType::I32
        | wasmtime::ValType::I64
        | wasmtime::ValType::F32
        | wasmtime::ValType::F64 => {
          // Supported types
        }
        unsupported => {
          return Err(ProviderError::InvocationFailed(format!(
            "WASM function '{}' in plugin '{}' returns unsupported type: {:?}. Only I32, I64, F32, F64 are supported.",
            _function, _plugin_name, unsupported
          )));
        }
      }
    }

    // Create result buffer with correct types for each return value
    let mut results: Vec<Val> = func_type
      .results()
      .map(|val_type| match val_type {
        wasmtime::ValType::I32 => Val::I32(0),
        wasmtime::ValType::I64 => Val::I64(0),
        wasmtime::ValType::F32 => Val::F32(0),
        wasmtime::ValType::F64 => Val::F64(0),
        _ => unreachable!("Validated above"),
      })
      .collect();

    // Call the function
    func
      .call(&mut plugin.store, &wasm_args, &mut results)
      .map_err(|e| {
        let error_msg = format!(
          "Failed to call function '{}' in plugin '{}': {}",
          _function, _plugin_name, e
        );
        error!("{}", error_msg);
        ProviderError::InvocationFailed(error_msg)
      })?;

    // Convert result back to ProviderValue
    if results.is_empty() {
      Ok(ProviderValue::Null)
    } else if results.len() == 1 {
      val_to_provider_value(&results[0])
    } else {
      // Multiple return values -> convert to Array
      let converted: Result<Vec<ProviderValue>, ProviderError> =
        results.iter().map(val_to_provider_value).collect();
      Ok(ProviderValue::Array(converted?))
    }
  }

  fn unload(&self, _instance: Self::Instance) -> Result<(), ProviderError> {
    // Implementation for unloading the WASM store
    todo!()
  }
}
