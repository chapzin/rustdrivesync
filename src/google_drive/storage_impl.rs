// Implementação do trait StorageBackend para Google Drive

use crate::error::{Result, RustDriveSyncError};
use crate::google_drive::client::DriveClient;
use crate::google_drive::models::ListFilesFilter;
use crate::google_drive::rate_limiter::{DriveRateLimiter, RateLimitedClient};
use crate::google_drive::upload::DriveUploader;
use crate::storage::{FileInfo, FolderInfo, StorageBackend, StorageStats, UploadOptions, UploadResult};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

/// Wrapper do DriveClient que implementa StorageBackend com rate limiting
pub struct DriveStorageBackend {
    client: RateLimitedClient<DriveClient>,
}

impl DriveStorageBackend {
    /// Cria um novo backend de storage com Google Drive
    ///
    /// # Argumentos
    /// * `client` - Cliente do Google Drive autenticado
    pub fn new(client: DriveClient) -> Self {
        let limiter = DriveRateLimiter::new();
        Self {
            client: RateLimitedClient::new(client, limiter),
        }
    }

    /// Cria um backend com rate limiter customizado
    ///
    /// # Argumentos
    /// * `client` - Cliente do Google Drive autenticado
    /// * `limiter` - Rate limiter configurado
    pub fn with_rate_limiter(client: DriveClient, limiter: DriveRateLimiter) -> Self {
        Self {
            client: RateLimitedClient::new(client, limiter),
        }
    }

    /// Obtém referência ao cliente interno
    pub fn client(&self) -> &DriveClient {
        self.client.inner()
    }

    /// Obtém referência ao rate limiter
    pub fn rate_limiter(&self) -> &DriveRateLimiter {
        self.client.limiter()
    }
}

#[async_trait]
impl StorageBackend for DriveStorageBackend {
    async fn upload_file(
        &self,
        file_path: &Path,
        options: UploadOptions,
    ) -> Result<UploadResult> {
        debug!("Iniciando upload de: {}", file_path.display());

        let _guard = self.client.limiter().acquire().await;
        let client = self.client.inner();

        let uploader = DriveUploader::new(client);

        // Converter opções do storage para opções do Drive
        let drive_options = crate::google_drive::models::UploadOptions {
            parent_folder_id: options.parent_folder_id.clone(),
            use_resumable: options.use_resumable,
            verify_checksum: options.verify_checksum,
            chunk_size: 256 * 1024, // 256KB default
            progress_callback: None,
        };

        let result = uploader.upload_file(file_path, drive_options).await?;

        // Converter resultado do Drive para resultado do storage
        Ok(UploadResult {
            file_id: result.file_id,
            name: result.name,
            size: result.size,
            md5_checksum: result.md5_checksum,
            web_view_link: result.web_view_link,
            upload_duration_secs: result.upload_duration_secs,
        })
    }

    async fn ensure_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo> {
        debug!("Garantindo que pasta '{}' existe", name);

        let _guard = self.client.limiter().acquire().await;
        let client = self.client.inner();

        let folder = client.ensure_folder(name, parent_id).await?;

        Ok(FolderInfo {
            id: folder.id,
            name: folder.name,
            parent_id: folder.parent_id,
        })
    }

    async fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<FileInfo>> {
        debug!("Listando arquivos na pasta: {:?}", folder_id);

        let _guard = self.client.limiter().acquire().await;
        let client = self.client.inner();

        let filter = ListFilesFilter {
            parent_folder_id: folder_id.map(|s| s.to_string()),
            name_contains: None,
            mime_type: None,
            max_results: None,
        };

        let files = client.list_files(filter).await?;

        Ok(files
            .into_iter()
            .map(|f| FileInfo {
                id: f.id.unwrap_or_default(),
                name: f.name.unwrap_or_default(),
                size: f.size.unwrap_or(0) as u64,
                mime_type: f.mime_type.unwrap_or_default(),
                modified_time: f.modified_time.map(|t| {
                    t.format("%s")
                        .to_string()
                        .parse()
                        .unwrap_or(0)
                }),
                md5_checksum: f.md5_checksum,
            })
            .collect())
    }

    async fn file_exists(
        &self,
        name: &str,
        parent_id: Option<String>,
    ) -> Result<Option<String>> {
        debug!("Verificando se arquivo '{}' existe", name);

        let _guard = self.client.limiter().acquire().await;
        let client = self.client.inner();

        client.file_exists(name, parent_id).await
    }

    async fn get_stats(&self) -> Result<StorageStats> {
        debug!("Obtendo estatísticas de storage");

        let _guard = self.client.limiter().acquire().await;
        let client = self.client.inner();

        // Usar API About do Google Drive para obter quota
        let (_, about) = client
            .hub()
            .about()
            .get()
            .param("fields", "storageQuota")
            .doit()
            .await
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao obter informações de quota: {}", e),
            })?;

        let storage_quota = about.storage_quota.ok_or_else(|| {
            RustDriveSyncError::DriveApiError {
                message: "Informações de quota não disponíveis".to_string(),
            }
        })?;

        let used = storage_quota.usage.unwrap_or(0) as u64;
        let limit = storage_quota.limit.map(|l| l as u64);

        Ok(StorageStats::new(used, limit))
    }

    async fn get_token(&self) -> Result<String> {
        let _guard = self.client.limiter().acquire().await;
        let client = self.client.inner();

        client.get_token().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_storage_backend_creation() {
        // Teste básico de compilação
        // Testes reais requerem credenciais do Google
    }
}
