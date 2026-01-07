// Modelos compartilhados para storage backends

use serde::{Deserialize, Serialize};

/// Opções para upload de arquivo
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    /// ID da pasta de destino
    pub parent_folder_id: Option<String>,
    /// Se deve usar upload resumível (para arquivos grandes)
    pub use_resumable: bool,
    /// Se deve verificar checksum após upload
    pub verify_checksum: bool,
}

impl UploadOptions {
    /// Cria opções com pasta pai especificada
    pub fn with_parent(parent_id: String) -> Self {
        Self {
            parent_folder_id: Some(parent_id),
            use_resumable: true,
            verify_checksum: true,
        }
    }

    /// Cria opções para upload resumível
    pub fn resumable() -> Self {
        Self {
            use_resumable: true,
            verify_checksum: true,
            ..Default::default()
        }
    }
}

/// Resultado de um upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// ID do arquivo no storage
    pub file_id: String,
    /// Nome do arquivo
    pub name: String,
    /// Tamanho em bytes
    pub size: u64,
    /// Checksum MD5 (se disponível)
    pub md5_checksum: Option<String>,
    /// Link para visualização web (se disponível)
    pub web_view_link: Option<String>,
    /// Duração do upload em segundos
    pub upload_duration_secs: f64,
}

/// Informações de uma pasta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    /// ID da pasta
    pub id: String,
    /// Nome da pasta
    pub name: String,
    /// ID da pasta pai (None se for raiz)
    pub parent_id: Option<String>,
}

/// Informações de um arquivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// ID do arquivo
    pub id: String,
    /// Nome do arquivo
    pub name: String,
    /// Tamanho em bytes
    pub size: u64,
    /// Tipo MIME
    pub mime_type: String,
    /// Timestamp de modificação
    pub modified_time: Option<i64>,
    /// Checksum MD5
    pub md5_checksum: Option<String>,
}

/// Estatísticas de uso do storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Espaço usado em bytes
    pub used_bytes: u64,
    /// Limite total em bytes (None se ilimitado)
    pub limit_bytes: Option<u64>,
    /// Percentual usado (0-100)
    pub usage_percentage: f64,
}

impl StorageStats {
    /// Cria estatísticas calculando o percentual
    pub fn new(used_bytes: u64, limit_bytes: Option<u64>) -> Self {
        let usage_percentage = if let Some(limit) = limit_bytes {
            if limit > 0 {
                (used_bytes as f64 / limit as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Self {
            used_bytes,
            limit_bytes,
            usage_percentage,
        }
    }

    /// Verifica se está próximo do limite (>90%)
    pub fn is_near_limit(&self) -> bool {
        self.usage_percentage > 90.0
    }

    /// Verifica se excedeu o limite
    pub fn is_over_limit(&self) -> bool {
        self.usage_percentage >= 100.0
    }
}
