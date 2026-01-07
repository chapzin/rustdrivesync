// Módulo de abstração de storage backend
pub mod models;

use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

pub use models::{FileInfo, FolderInfo, StorageStats, UploadOptions, UploadResult};

/// Trait abstrato para backends de armazenamento em nuvem
///
/// Permite implementações para diferentes provedores (Google Drive, S3, Dropbox, etc)
/// seguindo o princípio de Dependency Inversion (SOLID).
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Faz upload de um arquivo para o storage
    ///
    /// # Argumentos
    /// * `file_path` - Caminho do arquivo local
    /// * `options` - Opções de upload (pasta destino, verificação, etc)
    ///
    /// # Retorna
    /// Resultado do upload com ID do arquivo, MD5, links, etc
    async fn upload_file(
        &self,
        file_path: &Path,
        options: UploadOptions,
    ) -> Result<UploadResult>;

    /// Garante que uma pasta existe, criando se necessário
    ///
    /// # Argumentos
    /// * `name` - Nome da pasta
    /// * `parent_id` - ID da pasta pai (None para raiz)
    ///
    /// # Retorna
    /// Informações da pasta (ID, nome, pai)
    async fn ensure_folder(&self, name: &str, parent_id: Option<String>) -> Result<FolderInfo>;

    /// Lista arquivos em uma pasta
    ///
    /// # Argumentos
    /// * `folder_id` - ID da pasta (None para raiz)
    ///
    /// # Retorna
    /// Lista de informações de arquivos
    async fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<FileInfo>>;

    /// Verifica se um arquivo existe
    ///
    /// # Argumentos
    /// * `name` - Nome do arquivo
    /// * `parent_id` - ID da pasta pai
    ///
    /// # Retorna
    /// Some(file_id) se existe, None caso contrário
    async fn file_exists(&self, name: &str, parent_id: Option<String>)
        -> Result<Option<String>>;

    /// Obtém estatísticas de uso do storage
    ///
    /// # Retorna
    /// Estatísticas (espaço usado, limite, etc)
    async fn get_stats(&self) -> Result<StorageStats>;

    /// Obtém um token de acesso válido (para APIs que requerem)
    ///
    /// # Retorna
    /// Token de acesso como string
    async fn get_token(&self) -> Result<String>;
}
