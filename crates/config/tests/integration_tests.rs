use config::load_config;
use std::env;
use std::path::PathBuf;

fn get_fixture_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
}

#[test]
fn test_parse_valid_config() {
  let fixture_dir = get_fixture_dir();

  let original_dir = env::current_dir().unwrap();
  env::set_current_dir(&fixture_dir).unwrap();

  let result = load_config();
  assert!(result.is_ok());
  let config = result.unwrap();
  assert_eq!(config.name, "bud");
  assert_eq!(config.version, "0.1.0");
  assert_eq!(config.description, "A test configuration");

  env::set_current_dir(original_dir).unwrap();
}
