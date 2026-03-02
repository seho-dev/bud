//! Integration tests for PluginManager
//!
//! These tests are placed in crates/core/test/ as integration tests.
//! They can only access public APIs of the core crate.

use core::plugin::PluginManager;
use shared_types::ProviderValue;
use shared_types::config::ConfigData;
use shared_types::plugin::PluginError;
use std::sync::{Arc, Once};
use wasm_provider::WasmProvider;

static TEST_PLUGIN_SETUP: Once = Once::new();

fn create_manager() -> PluginManager<WasmProvider> {
  let config = Arc::new(ConfigData {
    name: "test-app".to_string(),
    version: "0.1.0".to_string(),
    description: "A test application".to_string(),
  });
  let provider = Arc::new(WasmProvider::new());
  PluginManager::new(config, provider).unwrap()
}

fn setup_test_plugin_once() {
  TEST_PLUGIN_SETUP.call_once(|| {
    let mut manager = create_manager();
    let path = workspace_root::get_workspace_root().join("example/sum-plugin");
    let target = manager.project_data_path().join("sum-plugin");

    if target.exists() {
      std::fs::remove_dir_all(&target).expect("Failed to remove stale test plugin directory");
    }

    manager
      .install(&path)
      .expect("Failed to install test plugin during one-time setup");
  });
}

#[test]
fn test_plugin_install() {
  let manager = create_manager();
  setup_test_plugin_once();
  assert!(manager.project_data_path().join("sum-plugin").exists());
}

#[test]
fn test_plugin_manager_new() {
  let manager = create_manager();
  assert_eq!(manager.config().name, "test-app");
  assert_eq!(manager.config().version, "0.1.0");
  assert_eq!(manager.config().description, "A test application");
}

#[test]
fn test_plugin_manager_get_all() {
  let mut manager = create_manager();
  let result = manager.get_all();
  assert!(result.is_ok() || matches!(result, Err(PluginError::LoadError(_))));
}
#[test]
fn test_plugin_manager_get() {
  let mut manager = create_manager();
  setup_test_plugin_once();
  let result = manager.get("sum-plugin");
  assert!(result.is_ok());
}

#[test]
fn test_plugin_manager_load() {
  let mut manager = create_manager();
  manager.init().expect("Failed to initialize provider");
  setup_test_plugin_once();
  let result = manager.load("sum-plugin");
  assert!(result.is_ok());
}

#[test]
#[ignore]
fn test_plugin_manager_invoke() {
  let mut manager = create_manager();
  manager.init().expect("Failed to initialize provider");
  setup_test_plugin_once();
  manager.load("sum-plugin").expect("Failed to load plugin");

  let result = manager.invoke(
    "sum-plugin",
    "Sum",
    vec![ProviderValue::Int(1), ProviderValue::Int(2)],
  );

  assert!(
    result.is_ok(),
    "Plugin invocation should succeed: {:?}",
    result
  );
  let value = result.unwrap();
  assert_eq!(value, ProviderValue::Int(3), "Sum(1, 2) should return 3");
}
