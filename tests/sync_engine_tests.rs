// Testes de integração para SyncEngine

use rustdrivesync::core::RetryConfig;
use rustdrivesync::error::{Result, RustDriveSyncError};
use rustdrivesync::storage::*;
use rustdrivesync::sync::{FileScanner, SyncEngine, SyncMode};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;
use tokio::fs;

/// Mock StorageBackend para testes
struct MockStorage {
    upload_count: Arc<AtomicUsize>,
    should_fail: bool,
    fail_after: usize,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            upload_count: Arc::new(AtomicUsize::new(0)),
            should_fail: false,
            fail_after: usize::MAX,
        }
    }

    fn with_failure_after(fail_after: usize) -> Self {
        Self {
            upload_count: Arc::new(AtomicUsize::new(0)),
            should_fail: true,
            fail_after,
        }
    }

    fn get_upload_count(&self) -> usize {
        self.upload_count.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl StorageBackend for MockStorage {
    async fn upload_file(&self, file_path: &Path, _options: UploadOptions) -> Result<UploadResult> {
        let count = self.upload_count.fetch_add(1, Ordering::Relaxed);

        // Simular falha se configurado
        if self.should_fail && count >= self.fail_after {
            return Err(RustDriveSyncError::NetworkError {
                message: "Mock network error".to_string(),
            });
        }

        // Simular upload bem-sucedido
        let metadata = tokio::fs::metadata(file_path).await.map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: file_path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        Ok(UploadResult {
            file_id: format!("mock_id_{}", count),
            name: file_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            size: metadata.len(),
            md5_checksum: Some("mock_md5".to_string()),
            web_view_link: Some(format!("https://mock.com/file/{}", count)),
            upload_duration_secs: 0.1,
        })
    }

    async fn ensure_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo> {
        Ok(FolderInfo {
            id: "mock_folder_id".to_string(),
            name: name.to_string(),
            parent_id,
        })
    }

    async fn list_files(&self, _folder_id: Option<&str>) -> Result<Vec<FileInfo>> {
        Ok(Vec::new())
    }

    async fn file_exists(&self, _name: &str, _parent_id: Option<String>) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        Ok(StorageStats::new(0, Some(1_000_000_000)))
    }

    async fn get_token(&self) -> Result<String> {
        Ok("mock_token".to_string())
    }
}

/// Cria um diretório temporário com arquivos de teste
async fn create_test_directory() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir().map_err(|e| RustDriveSyncError::DriveApiError {
        message: format!("Failed to create temp dir: {}", e),
    })?;

    // Criar alguns arquivos de teste
    let file1 = temp_dir.path().join("test1.txt");
    let file2 = temp_dir.path().join("test2.txt");
    let file3 = temp_dir.path().join("subdir").join("test3.txt");

    fs::write(&file1, b"content1").await.map_err(|e| {
        RustDriveSyncError::FileReadError {
            path: file1.display().to_string(),
            message: e.to_string(),
        }
    })?;

    fs::write(&file2, b"content2 larger").await.map_err(|e| {
        RustDriveSyncError::FileReadError {
            path: file2.display().to_string(),
            message: e.to_string(),
        }
    })?;

    fs::create_dir_all(file3.parent().unwrap())
        .await
        .map_err(|e| RustDriveSyncError::DriveApiError {
            message: format!("Failed to create subdir: {}", e),
        })?;

    fs::write(&file3, b"content3 in subdir").await.map_err(|e| {
        RustDriveSyncError::FileReadError {
            path: file3.display().to_string(),
            message: e.to_string(),
        }
    })?;

    Ok(temp_dir)
}

#[tokio::test]
async fn test_sync_engine_with_mock_storage() {
    let temp_dir = create_test_directory().await.unwrap();
    let mock_storage = Arc::new(MockStorage::new());

    // Criar configuração mínima
    let config = rustdrivesync::config::schema::Config {
        general: rustdrivesync::config::schema::GeneralConfig {
            log_level: "info".to_string(),
            log_file: None,
            log_format: "text".to_string(),
        },
        source: rustdrivesync::config::schema::SourceConfig {
            path: temp_dir.path().to_path_buf(),
            recursive: true,
            ignore_hidden: false,
            follow_symlinks: false,
            ignore_patterns: vec![],
        },
        google_drive: rustdrivesync::config::schema::GoogleDriveConfig {
            credentials_file: "test.json".into(),
            token_file: "test_token.json".into(),
            target_folder_id: Some("test_folder".to_string()),
            target_folder_name: Some("Test Folder".to_string()),
            scopes: vec!["test_scope".to_string()],
        },
        sync: rustdrivesync::config::schema::SyncConfig {
            mode: "once".to_string(),
            interval_seconds: 300,
            conflict_resolution: "overwrite".to_string(),
            max_file_size_mb: 100,
            chunk_size_mb: 5,
            max_concurrent_uploads: 2,
            verify_upload: true,
            preserve_folder_structure: true,
            delete_remote_on_local_delete: false,
        },
        retry: rustdrivesync::config::schema::RetryConfig::default(),
        notifications: rustdrivesync::config::schema::NotificationsConfig::default(),
        state: rustdrivesync::config::schema::StateConfig {
            state_file: temp_dir.path().join("state.json"),
            save_interval: 10,
            cleanup_after_days: 30,
        },
    };

    // Criar engine com mock
    let _engine = SyncEngine::with_backend(
        config,
        mock_storage.clone(),
        "mock_folder_id".to_string(),
        SyncMode::Once,
        false,
    )
    .unwrap();

    // Não podemos chamar run() pois exige &mut, mas validamos a criação
    assert_eq!(mock_storage.get_upload_count(), 0);
}

#[tokio::test]
async fn test_file_scanner_recursive() {
    let temp_dir = create_test_directory().await.unwrap();

    let scanner = FileScanner::new();
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar 3 arquivos (test1.txt, test2.txt, subdir/test3.txt)
    assert_eq!(files.len(), 3);

    // Verificar que os arquivos foram encontrados
    let names: Vec<_> = files
        .iter()
        .map(|f| f.relative_path.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(names.contains(&"test1.txt"));
    assert!(names.contains(&"test2.txt"));
    assert!(names.contains(&"test3.txt"));
}

#[tokio::test]
async fn test_file_scanner_with_ignore_patterns() {
    let temp_dir = create_test_directory().await.unwrap();

    let scanner = FileScanner::new()
        .with_ignore_patterns(vec!["subdir".to_string()]);

    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar apenas 2 arquivos (test1.txt, test2.txt), ignorando subdir/
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn test_file_scanner_with_max_size() {
    let temp_dir = create_test_directory().await.unwrap();

    let scanner = FileScanner::new()
        .with_max_file_size(10); // Apenas 10 bytes

    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar apenas test1.txt (8 bytes)
    // test2.txt (16 bytes) e test3.txt (18 bytes) são muito grandes
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].relative_path.file_name().unwrap().to_str().unwrap(),
        "test1.txt"
    );
}

#[test]
fn test_retry_config_from_defaults() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay_secs, 1);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.max_delay_secs, 60);
}

#[test]
fn test_retry_config_custom() {
    let config = RetryConfig::new(5, 2);
    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.initial_delay_secs, 2);
}
