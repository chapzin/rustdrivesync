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

    /// Garante que toda uma hierarquia de pastas existe, criando recursivamente se necessário
    ///
    /// # Argumentos
    /// * `path` - Caminho relativo das pastas (ex: "projeto/subpasta/docs")
    /// * `root_folder_id` - ID da pasta raiz onde criar a hierarquia
    ///
    /// # Retorna
    /// Informações da pasta final (mais profunda na hierarquia)
    ///
    /// # Exemplo
    /// ```ignore
    /// // Cria: root/projeto/subpasta/docs
    /// let folder = backend.ensure_folder_path("projeto/subpasta/docs", root_id).await?;
    /// // folder.id = ID da pasta "docs"
    /// ```
    async fn ensure_folder_path(&self, path: &str, root_folder_id: String) -> Result<FolderInfo> {
        use std::path::Path;

        let path_obj = Path::new(path);
        let mut current_parent_id = root_folder_id;
        let mut last_folder: Option<FolderInfo> = None;

        // Iterar sobre cada componente do caminho
        for component in path_obj.components() {
            if let Some(name) = component.as_os_str().to_str() {
                // Pular componentes vazios ou "."
                if name.is_empty() || name == "." {
                    continue;
                }

                // Criar/garantir que esta pasta existe
                let folder = self.ensure_folder(name, Some(current_parent_id.clone())).await?;

                // A próxima pasta será filha desta
                current_parent_id = folder.id.clone();
                last_folder = Some(folder);
            }
        }

        // Retornar a última pasta criada (mais profunda)
        last_folder.ok_or_else(|| crate::error::RustDriveSyncError::DriveApiError {
            message: format!("Nenhuma pasta foi criada a partir do caminho: {}", path),
        })
    }

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
