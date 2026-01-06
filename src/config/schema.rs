use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuração principal do RustDriveSync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub source: SourceConfig,
    pub google_drive: GoogleDriveConfig,
    pub sync: SyncConfig,
    #[serde(default)]
    pub retry: RetryConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
    #[serde(default)]
    pub state: StateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub log_file: Option<PathBuf>,
    #[serde(default = "default_log_format")]
    pub log_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub path: PathBuf,
    #[serde(default = "default_true")]
    pub recursive: bool,
    #[serde(default = "default_true")]
    pub ignore_hidden: bool,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveConfig {
    pub credentials_file: PathBuf,
    pub token_file: PathBuf,
    pub target_folder_id: Option<String>,
    pub target_folder_name: Option<String>,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_sync_mode")]
    pub mode: String,
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    #[serde(default = "default_conflict_resolution")]
    pub conflict_resolution: String,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    #[serde(default = "default_chunk_size")]
    pub chunk_size_mb: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_uploads: usize,
    #[serde(default = "default_true")]
    pub verify_upload: bool,
    #[serde(default = "default_true")]
    pub preserve_folder_structure: bool,
    #[serde(default)]
    pub delete_remote_on_local_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_initial_delay")]
    pub initial_delay_seconds: u64,
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    #[serde(default = "default_max_delay")]
    pub max_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub on_complete: bool,
    #[serde(default = "default_true")]
    pub on_error: bool,
    #[serde(default = "default_notification_type")]
    pub notification_type: String,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    #[serde(default = "default_state_file")]
    pub state_file: PathBuf,
    #[serde(default = "default_save_interval")]
    pub save_interval: u64,
    #[serde(default = "default_cleanup_days")]
    pub cleanup_after_days: u64,
}

// Valores padrão
fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_true() -> bool {
    true
}

fn default_sync_mode() -> String {
    "once".to_string()
}

fn default_interval() -> u64 {
    300
}

fn default_conflict_resolution() -> String {
    "overwrite".to_string()
}

fn default_max_file_size() -> u64 {
    100
}

fn default_chunk_size() -> u64 {
    5
}

fn default_max_concurrent() -> usize {
    4
}

fn default_scopes() -> Vec<String> {
    vec!["https://www.googleapis.com/auth/drive.file".to_string()]
}

fn default_max_attempts() -> u32 {
    3
}

fn default_initial_delay() -> u64 {
    1
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_max_delay() -> u64 {
    60
}

fn default_notification_type() -> String {
    "desktop".to_string()
}

fn default_state_file() -> PathBuf {
    PathBuf::from("./state.json")
}

fn default_save_interval() -> u64 {
    10
}

fn default_cleanup_days() -> u64 {
    30
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            initial_delay_seconds: default_initial_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            max_delay_seconds: default_max_delay(),
        }
    }
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            on_complete: true,
            on_error: true,
            notification_type: default_notification_type(),
            webhook_url: None,
        }
    }
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            state_file: default_state_file(),
            save_interval: default_save_interval(),
            cleanup_after_days: default_cleanup_days(),
        }
    }
}
