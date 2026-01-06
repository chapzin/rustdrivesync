use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Resultado de um upload para o Google Drive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// ID do arquivo no Google Drive
    pub file_id: String,
    /// Nome do arquivo
    pub name: String,
    /// Tamanho do arquivo em bytes
    pub size: u64,
    /// Hash MD5 do arquivo
    pub md5_checksum: Option<String>,
    /// URL de visualização do arquivo
    pub web_view_link: Option<String>,
    /// Tempo que levou para fazer upload (em segundos)
    pub upload_duration_secs: f64,
}

/// Metadados de um arquivo local a ser enviado
#[derive(Debug, Clone)]
pub struct LocalFileMetadata {
    /// Caminho completo do arquivo local
    pub path: PathBuf,
    /// Nome do arquivo (pode ser diferente do path)
    pub name: String,
    /// Tamanho do arquivo em bytes
    pub size: u64,
    /// Tipo MIME do arquivo
    pub mime_type: String,
    /// Hash MD5 do arquivo (calculado localmente)
    pub md5_hash: Option<String>,
}

/// Informações de progresso de upload
#[derive(Debug, Clone)]
pub struct UploadProgress {
    /// Bytes enviados até agora
    pub bytes_uploaded: u64,
    /// Total de bytes a enviar
    pub total_bytes: u64,
    /// Porcentagem concluída (0-100)
    pub percentage: f32,
    /// Velocidade média em bytes/segundo
    pub bytes_per_second: f64,
}

impl UploadProgress {
    /// Cria um novo progresso de upload
    pub fn new(bytes_uploaded: u64, total_bytes: u64) -> Self {
        let percentage = if total_bytes > 0 {
            (bytes_uploaded as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        Self {
            bytes_uploaded,
            total_bytes,
            percentage,
            bytes_per_second: 0.0,
        }
    }

    /// Verifica se o upload está completo
    pub fn is_complete(&self) -> bool {
        self.bytes_uploaded >= self.total_bytes
    }
}

/// Opções para upload de arquivo
pub struct UploadOptions {
    /// ID da pasta de destino no Google Drive
    pub parent_folder_id: Option<String>,
    /// Se deve usar upload resumível (para arquivos grandes)
    pub use_resumable: bool,
    /// Tamanho do chunk para upload resumível (em bytes)
    /// Deve ser múltiplo de 256KB
    pub chunk_size: usize,
    /// Se deve verificar MD5 após upload
    pub verify_checksum: bool,
    /// Callback para progresso (opcional)
    pub progress_callback: Option<Box<dyn Fn(UploadProgress) + Send + Sync>>,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            parent_folder_id: None,
            use_resumable: true,
            chunk_size: 5 * 1024 * 1024, // 5MB
            verify_checksum: true,
            progress_callback: None,
        }
    }
}

impl UploadOptions {
    /// Cria opções com pasta de destino
    pub fn with_parent(parent_folder_id: String) -> Self {
        Self {
            parent_folder_id: Some(parent_folder_id),
            ..Default::default()
        }
    }

    /// Define o tamanho do chunk (deve ser múltiplo de 256KB)
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        // Garantir que é múltiplo de 256KB
        const CHUNK_MULTIPLE: usize = 256 * 1024;
        self.chunk_size = (size / CHUNK_MULTIPLE) * CHUNK_MULTIPLE;
        if self.chunk_size == 0 {
            self.chunk_size = CHUNK_MULTIPLE;
        }
        self
    }

    /// Define se deve usar upload resumível
    pub fn with_resumable(mut self, resumable: bool) -> Self {
        self.use_resumable = resumable;
        self
    }
}

/// Informações de uma pasta no Google Drive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    /// ID da pasta
    pub id: String,
    /// Nome da pasta
    pub name: String,
    /// ID da pasta pai (None se for raiz)
    pub parent_id: Option<String>,
}

/// Filtros para listagem de arquivos
#[derive(Debug, Clone, Default)]
pub struct ListFilesFilter {
    /// Filtrar por pasta pai
    pub parent_folder_id: Option<String>,
    /// Filtrar por nome (busca parcial)
    pub name_contains: Option<String>,
    /// Filtrar por tipo MIME
    pub mime_type: Option<String>,
    /// Número máximo de resultados
    pub max_results: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_progress_calculation() {
        let progress = UploadProgress::new(50, 100);
        assert_eq!(progress.percentage, 50.0);
        assert!(!progress.is_complete());

        let complete = UploadProgress::new(100, 100);
        assert_eq!(complete.percentage, 100.0);
        assert!(complete.is_complete());
    }

    #[test]
    fn test_chunk_size_rounding() {
        let options = UploadOptions::default().with_chunk_size(1_000_000);
        // Deve arredondar para múltiplo de 256KB (768KB)
        assert_eq!(options.chunk_size % (256 * 1024), 0);
    }
}
