use config::load_plugin_config;
use shared_types::config::{
  EnvPermission, FilesystemPermission, NetworkPermission, Permission, Permissions,
  ProcessPermission, StdioPermission,
};
use std::fs;
use tempfile::TempDir;

// Helper: write a plugin.json with the given permissions JSON fragment into a temp dir,
// then load it via load_plugin_config and return the permissions field.
fn load_permissions(permissions_json: &str) -> Option<Permissions> {
  let dir = TempDir::new().expect("failed to create temp dir");
  let plugin_json = format!(
    r#"{{
      "name": "test-plugin",
      "version": "1.0.0",
      "description": "Test plugin",
      "author": "tester",
      "permissions": {}
    }}"#,
    permissions_json
  );
  fs::write(dir.path().join("plugin.json"), plugin_json).expect("failed to write plugin.json");
  let config = load_plugin_config(dir.path()).expect("load_plugin_config failed");
  config.permissions
}

// case 1: stdio: true → Permission::Bool(true)
#[test]
fn test_stdio_bool_true() {
  let perms = load_permissions(r#"{"stdio": true}"#).unwrap();
  assert_eq!(perms.stdio, Some(Permission::Bool(true)));
}

// case 2: stdio: false → Permission::Bool(false)
#[test]
fn test_stdio_bool_false() {
  let perms = load_permissions(r#"{"stdio": false}"#).unwrap();
  assert_eq!(perms.stdio, Some(Permission::Bool(false)));
}

// case 3: stdio as a detailed config object → Permission::Config(StdioPermission { ... })
#[test]
fn test_stdio_config_object() {
  let perms =
    load_permissions(r#"{"stdio": {"stdin": true, "stdout": true, "stderr": false}}"#).unwrap();
  assert_eq!(
    perms.stdio,
    Some(Permission::Config(StdioPermission {
      stdin: Some(true),
      stdout: Some(true),
      stderr: Some(false),
    }))
  );
}

// case 4: permissions field entirely absent → None
#[test]
fn test_permissions_field_absent() {
  let dir = TempDir::new().expect("failed to create temp dir");
  let plugin_json = r#"{
    "name": "test-plugin",
    "version": "1.0.0",
    "description": "Test plugin",
    "author": "tester"
  }"#;
  fs::write(dir.path().join("plugin.json"), plugin_json).expect("failed to write plugin.json");
  let config = load_plugin_config(dir.path()).expect("load_plugin_config failed");
  assert_eq!(config.permissions, None);
}

// case 5: filesystem as a detailed config object
#[test]
fn test_filesystem_config_object() {
  let perms =
    load_permissions(r#"{"filesystem": {"read": ["./config"], "write": ["./output"]}}"#).unwrap();
  assert_eq!(
    perms.filesystem,
    Some(Permission::Config(FilesystemPermission {
      read: Some(vec!["./config".to_string()]),
      write: Some(vec!["./output".to_string()]),
    }))
  );
}

// case 6: network as a detailed config object
#[test]
fn test_network_config_object() {
  let perms =
    load_permissions(r#"{"network": {"allowed_hosts": ["api.example.com:443"]}}"#).unwrap();
  assert_eq!(
    perms.network,
    Some(Permission::Config(NetworkPermission {
      allowed_hosts: Some(vec!["api.example.com:443".to_string()]),
    }))
  );
}

// case 7: network: true → Permission::Bool(true)
#[test]
fn test_network_bool_true() {
  let perms = load_permissions(r#"{"network": true}"#).unwrap();
  assert_eq!(perms.network, Some(Permission::Bool(true)));
}

// case 8: env as a detailed config object
#[test]
fn test_env_config_object() {
  let perms = load_permissions(r#"{"env": {"inherit": false, "keys": ["APP_ENV"]}}"#).unwrap();
  assert_eq!(
    perms.env,
    Some(Permission::Config(EnvPermission {
      inherit: Some(false),
      keys: Some(vec!["APP_ENV".to_string()]),
    }))
  );
}

// case 9: process as a detailed config object
#[test]
fn test_process_config_object() {
  let perms = load_permissions(r#"{"process": {"exit": false}}"#).unwrap();
  assert_eq!(
    perms.process,
    Some(Permission::Config(ProcessPermission { exit: Some(false) }))
  );
}

// case 10: permissions as an empty object → Permissions::default() (all fields None)
#[test]
fn test_permissions_empty_object() {
  let perms = load_permissions("{}").unwrap();
  assert_eq!(perms, Permissions::default());
}

// case 11: invalid permission value type → Schema validation should reject it
#[test]
fn test_invalid_stdio_type_rejected() {
  let dir = TempDir::new().expect("failed to create temp dir");
  let plugin_json = r#"{
    "name": "test-plugin",
    "version": "1.0.0",
    "description": "Test plugin",
    "author": "tester",
    "permissions": {"stdio": "invalid_string"}
  }"#;
  fs::write(dir.path().join("plugin.json"), plugin_json).expect("failed to write plugin.json");
  let result = load_plugin_config(dir.path());
  assert!(
    result.is_err(),
    "expected validation error for invalid stdio type"
  );
}
