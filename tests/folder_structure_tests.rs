//! Testes para preservação de estrutura de pastas

use rustdrivesync::storage::{FolderInfo, StorageBackend};
use rustdrivesync::error::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Mock simples para testar a criação de hierarquia
struct SimpleMockStorage {
    folders: Arc<Mutex<HashMap<String, FolderInfo>>>,
    call_count: Arc<Mutex<usize>>,
}

impl SimpleMockStorage {
    fn new() -> Self {
        Self {
            folders: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    async fn get_call_count(&self) -> usize {
        *self.call_count.lock().await
    }
}

#[async_trait::async_trait]
impl StorageBackend for SimpleMockStorage {
    async fn upload_file(
        &self,
        _file_path: &std::path::Path,
        _options: rustdrivesync::storage::UploadOptions,
    ) -> Result<rustdrivesync::storage::UploadResult> {
        unimplemented!("Not needed for folder tests")
    }

    async fn ensure_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo> {
        let mut count = self.call_count.lock().await;
        *count += 1;

        let folder_id = format!("folder_{}_{}", name, *count);
        let folder = FolderInfo {
            id: folder_id.clone(),
            name: name.to_string(),
            parent_id,
        };

        let mut folders = self.folders.lock().await;
        folders.insert(folder_id.clone(), folder.clone());

        Ok(folder)
    }

    async fn list_files(
        &self,
        _folder_id: Option<&str>,
    ) -> Result<Vec<rustdrivesync::storage::FileInfo>> {
        Ok(vec![])
    }

    async fn file_exists(&self, _name: &str, _parent_id: Option<String>) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_stats(&self) -> Result<rustdrivesync::storage::StorageStats> {
        Ok(rustdrivesync::storage::StorageStats {
            used_bytes: 0,
            limit_bytes: None,
            usage_percentage: 0.0,
        })
    }

    async fn get_token(&self) -> Result<String> {
        Ok("test_token".to_string())
    }
}

#[tokio::test]
async fn test_ensure_folder_path_single_level() {
    let storage = SimpleMockStorage::new();

    let result = storage
        .ensure_folder_path("projeto", "root_id".to_string())
        .await;

    assert!(result.is_ok());
    let folder = result.unwrap();
    assert_eq!(folder.name, "projeto");
    assert_eq!(folder.parent_id, Some("root_id".to_string()));

    // Deve ter chamado ensure_folder 1 vez
    assert_eq!(storage.get_call_count().await, 1);
}

#[tokio::test]
async fn test_ensure_folder_path_nested_two_levels() {
    let storage = SimpleMockStorage::new();

    let result = storage
        .ensure_folder_path("projeto/subpasta", "root_id".to_string())
        .await;

    assert!(result.is_ok());
    let folder = result.unwrap();
    assert_eq!(folder.name, "subpasta");

    // Deve ter chamado ensure_folder 2 vezes (projeto + subpasta)
    assert_eq!(storage.get_call_count().await, 2);
}

#[tokio::test]
async fn test_ensure_folder_path_deep_nested() {
    let storage = SimpleMockStorage::new();

    let result = storage
        .ensure_folder_path("a/b/c/d", "root_id".to_string())
        .await;

    assert!(result.is_ok());
    let folder = result.unwrap();
    assert_eq!(folder.name, "d");

    // Deve ter chamado ensure_folder 4 vezes (a, b, c, d)
    assert_eq!(storage.get_call_count().await, 4);
}

#[tokio::test]
async fn test_ensure_folder_path_with_empty_components() {
    let storage = SimpleMockStorage::new();

    // Caminho com componentes vazios ou apenas "."
    let result = storage
        .ensure_folder_path("./projeto/./subpasta", "root_id".to_string())
        .await;

    assert!(result.is_ok());
    let folder = result.unwrap();
    assert_eq!(folder.name, "subpasta");

    // Deve ignorar "." e chamar ensure_folder apenas 2 vezes
    assert_eq!(storage.get_call_count().await, 2);
}
