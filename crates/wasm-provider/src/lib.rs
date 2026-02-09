use log::info;
use shared_types::{Provider, ProviderError, ProviderValue};
use wasmtime::{Config, Engine};

#[derive(Debug, Default)]
pub struct WasmProvider;

impl WasmProvider {
  #[must_use]
  pub fn new() -> Self {
    Self
  }
}

impl Provider for WasmProvider {
  type Instance = Engine;
  const MAIN_FILE: &'static str = "main.wasm";

  fn init(&self) -> Result<Self::Instance, ProviderError> {
    info!("Initializing WasmProvider");

    // Create a new WASM engine with default configuration
    // Using Engine::new instead of Engine::default to properly handle configuration errors
    let config = Config::default();
    let engine = Engine::new(&config).map_err(|_| ProviderError::InitFailed)?;

    info!("WasmProvider initialized successfully");
    Ok(engine)
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
