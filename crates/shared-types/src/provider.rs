use std::path::Path;

/// Provider runtime error types.
///
/// Unified error types for all Provider implementations.
#[derive(thiserror::Error, Debug)]
pub enum ProviderError {
  /// Provider initialization failed.
  #[error("Init failed")]
  InitFailed,
  /// Provider load failed.
  #[error("Load failed")]
  LoadFailed(String),
  /// Function injection into runtime instance failed.
  #[error("Function injection failed: {0}")]
  InjectionFailed(String),
  /// Runtime function invocation failed.
  #[error("Function invocation failed: {0}")]
  InvocationFailed(String),
  /// Runtime instance unload failed.
  #[error("Instance unload failed: {0}")]
  UnloadFailed(String),
  /// Permission denied.
  #[error("Permission denied: {0}")]
  PermissionDenied(String),
}

/// Unified value type across different runtime environments.
///
/// Supports primitive and composite types for WASM, Bun, Node, etc.
#[derive(Debug, Clone)]
pub enum ProviderValue {
  /// Null value.
  Null,
  /// Boolean value.
  Bool(bool),
  /// 32-bit signed integer.
  I32(i32),
  /// 64-bit signed integer.
  I64(i64),
  /// 32-bit floating point.
  F32(f32),
  /// 64-bit floating point.
  F64(f64),
  /// String value.
  String(String),
  /// Array of values.
  Array(Vec<ProviderValue>),
  /// Object as key-value pairs.
  Object(Vec<(String, ProviderValue)>),
}

/// Cross-runtime provider abstraction.
///
/// Defines a unified interface for different runtime environments
/// like WASM, Bun, and Node.js.
///
/// # Implementation Requirements
///
/// - Must implement `Send + Sync` for thread safety
/// - Associated type `Instance` represents the provider's runtime instance
/// - Associated constant `MAIN_FILE` defines the main entry file
/// - All methods must return `Result` for error propagation
///
/// # Examples
///
/// ```
/// use shared_types::{Provider, ProviderError, ProviderValue};
///
/// struct MyProvider;
///
/// impl Provider for MyProvider {
///   type Instance = ();
///   const MAIN_FILE: &'static str = "main.js";
///
///   fn init(&self) -> Result<Self::Instance, ProviderError> {
///     Ok(())
///   }
///
///   fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), ProviderError> {
///     Ok(())
///   }
///
///   fn inject(
///     &self,
///     _instance: &mut Self::Instance,
///     _functions: &[(
///       &str,
///       &dyn Fn(Vec<ProviderValue>) -> Result<ProviderValue, ProviderError>,
///     )],
///   ) -> Result<(), ProviderError> {
///     Ok(())
///   }
///
///   fn invoke(
///     &self,
///     _instance: &mut Self::Instance,
///     _function: &str,
///     _args: Vec<ProviderValue>,
///   ) -> Result<ProviderValue, ProviderError> {
///     Ok(ProviderValue::Null)
///   }
///
///   fn unload(&self, _instance: Self::Instance) -> Result<(), ProviderError> {
///     Ok(())
///   }
/// }
/// ```
pub trait Provider: Send + Sync {
  /// Provider's internal runtime instance type.
  ///
  /// Different providers return different runtime instance types.
  /// Examples:
  /// - `WasmProvider` returns `wasmtime::Engine`
  /// - `BunProvider` returns `bun::JsGlobalObject`
  type Instance;

  /// Provider's main entry file (compile-time constant).
  ///
  /// Defines the main entry file loaded by the provider.
  /// Examples:
  /// - `WasmProvider::MAIN_FILE = "main.wasm"`
  /// - `BunProvider::MAIN_FILE = "main.js"`
  /// - `NodeProvider::MAIN_FILE = "index.js"`
  ///
  /// Benefits of using associated constants over methods:
  /// - Zero runtime overhead (inlined at compile time)
  /// - Can be used in generic constraints
  /// - Clearer semantics (explicitly constant)
  const MAIN_FILE: &'static str;

  /// Initialize the provider instance.
  ///
  /// Creates and returns the provider's internal runtime instance.
  ///
  /// # Errors
  ///
  /// Returns `ProviderError::InitFailed` if initialization fails.
  fn init(&self) -> Result<Self::Instance, ProviderError>;

  /// Load plugin using the provider.
  ///
  /// Loads the plugin and returns an error if it fails.
  ///
  /// # Errors
  ///
  /// Returns `ProviderError::LoadFailed` if loading fails.
  fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), ProviderError>;

  /// Inject host functions into the runtime.
  ///
  /// Exposes host environment functions to the guest runtime.
  ///
  /// # Arguments
  ///
  /// * `instance` - Provider's runtime instance
  /// * `functions` - List of functions to inject as (name, closure) tuples
  ///
  /// # Errors
  ///
  /// Returns `ProviderError::InjectionFailed` if injection fails.
  fn inject(
    &self,
    instance: &mut Self::Instance,
    functions: &[(
      &str,
      &dyn Fn(Vec<ProviderValue>) -> Result<ProviderValue, ProviderError>,
    )],
  ) -> Result<(), ProviderError>;

  /// Invoke a function in the runtime.
  ///
  /// Executes a named function in the runtime environment.
  ///
  /// # Arguments
  ///
  /// * `instance` - Provider's runtime instance
  /// * `function` - Function name to invoke
  /// * `args` - Function arguments
  ///
  /// # Errors
  ///
  /// Returns `ProviderError::InvocationFailed` if invocation fails.
  fn invoke(
    &self,
    instance: &mut Self::Instance,
    function: &str,
    args: Vec<ProviderValue>,
  ) -> Result<ProviderValue, ProviderError>;

  /// Unload the runtime instance.
  ///
  /// Releases all resources held by the provider instance.
  ///
  /// # Arguments
  ///
  /// * `instance` - Runtime instance to unload (takes ownership)
  ///
  /// # Errors
  ///
  /// Returns `ProviderError::UnloadFailed` if unload fails.
  fn unload(&self, instance: Self::Instance) -> Result<(), ProviderError>;
}
