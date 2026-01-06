use rustdrivesync::config::{load_config, validate_config};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let config_content = r#"
[general]
log_level = "info"

[source]
path = "."
recursive = true

[google_drive]
credentials_file = "credentials.json"
token_file = "token.json"
target_folder_id = "test123"

[sync]
mode = "once"
"#;

    fs::write(&config_path, config_content).unwrap();
    let config = load_config(&config_path).unwrap();

    assert_eq!(config.general.log_level, "info");
    assert_eq!(config.sync.mode, "once");
}

#[test]
fn test_load_nonexistent_config_fails() {
    let result = load_config("/nonexistent/path/config.toml");
    assert!(result.is_err());
}
