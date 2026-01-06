use crate::error::{Result, RustDriveSyncError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Estado de sincronização de um arquivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// Caminho relativo do arquivo
    pub relative_path: String,
    /// ID do arquivo no Google Drive
    pub drive_file_id: String,
    /// Tamanho do arquivo em bytes
    pub size: u64,
    /// Timestamp da última modificação
    pub modified: i64,
    /// Hash MD5 do arquivo
    pub md5_hash: String,
    /// Timestamp da última sincronização (Unix timestamp)
    pub last_synced: i64,
}

/// Estado geral da sincronização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Versão do formato de estado (para compatibilidade futura)
    pub version: String,
    /// Diretório local sendo sincronizado
    pub local_directory: String,
    /// ID da pasta raiz no Google Drive
    pub drive_folder_id: String,
    /// Timestamp da última sincronização completa
    pub last_full_sync: Option<i64>,
    /// Mapa de arquivos sincronizados (caminho relativo -> estado)
    pub files: HashMap<String, FileState>,
}

impl SyncState {
    /// Cria um novo estado de sincronização
    pub fn new(local_directory: String, drive_folder_id: String) -> Self {
        Self {
            version: "1.0".to_string(),
            local_directory,
            drive_folder_id,
            last_full_sync: None,
            files: HashMap::new(),
        }
    }

    /// Adiciona ou atualiza o estado de um arquivo
    pub fn update_file(&mut self, file_state: FileState) {
        self.files
            .insert(file_state.relative_path.clone(), file_state);
    }

    /// Remove um arquivo do estado
    pub fn remove_file(&mut self, relative_path: &str) -> Option<FileState> {
        self.files.remove(relative_path)
    }

    /// Obtém o estado de um arquivo
    pub fn get_file(&self, relative_path: &str) -> Option<&FileState> {
        self.files.get(relative_path)
    }

    /// Verifica se um arquivo foi sincronizado
    pub fn is_synced(&self, relative_path: &str) -> bool {
        self.files.contains_key(relative_path)
    }

    /// Marca a última sincronização completa
    pub fn mark_full_sync(&mut self) {
        self.last_full_sync = Some(chrono::Utc::now().timestamp());
    }

    /// Retorna o número de arquivos sincronizados
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// Gerenciador de estado de sincronização
pub struct SyncStateManager {
    /// Caminho do arquivo de estado
    state_file_path: PathBuf,
    /// Estado atual
    state: SyncState,
}

impl SyncStateManager {
    /// Cria um novo gerenciador de estado
    pub fn new<P: AsRef<Path>>(
        state_file_path: P,
        local_directory: String,
        drive_folder_id: String,
    ) -> Self {
        Self {
            state_file_path: state_file_path.as_ref().to_path_buf(),
            state: SyncState::new(local_directory, drive_folder_id),
        }
    }

    /// Carrega o estado do disco, ou cria um novo se não existir
    pub fn load_or_create(
        state_file_path: PathBuf,
        local_directory: String,
        drive_folder_id: String,
    ) -> Result<Self> {
        if state_file_path.exists() {
            info!("Carregando estado de: {}", state_file_path.display());
            Self::load(state_file_path)
        } else {
            info!("Criando novo estado em: {}", state_file_path.display());
            Ok(Self::new(state_file_path, local_directory, drive_folder_id))
        }
    }

    /// Carrega o estado do disco
    pub fn load<P: AsRef<Path>>(state_file_path: P) -> Result<Self> {
        let state_file_path = state_file_path.as_ref();

        let content = fs::read_to_string(state_file_path).map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: state_file_path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        let state: SyncState = serde_json::from_str(&content).map_err(|e| {
            RustDriveSyncError::ConfigInvalid {
                message: format!("Erro ao parsear estado: {}", e),
            }
        })?;

        debug!("Estado carregado: {} arquivos", state.file_count());

        Ok(Self {
            state_file_path: state_file_path.to_path_buf(),
            state,
        })
    }

    /// Salva o estado no disco
    pub fn save(&self) -> Result<()> {
        // Criar diretório pai se não existir
        if let Some(parent) = self.state_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| RustDriveSyncError::FileReadError {
                path: parent.display().to_string(),
                message: e.to_string(),
            })?;
        }

        let content = serde_json::to_string_pretty(&self.state).map_err(|e| {
            RustDriveSyncError::ConfigInvalid {
                message: format!("Erro ao serializar estado: {}", e),
            }
        })?;

        fs::write(&self.state_file_path, content).map_err(|e| {
            RustDriveSyncError::FileReadError {
                path: self.state_file_path.display().to_string(),
                message: e.to_string(),
            }
        })?;

        debug!("Estado salvo em: {}", self.state_file_path.display());
        Ok(())
    }

    /// Obtém referência imutável ao estado
    pub fn state(&self) -> &SyncState {
        &self.state
    }

    /// Obtém referência mutável ao estado
    pub fn state_mut(&mut self) -> &mut SyncState {
        &mut self.state
    }

    /// Atualiza o estado de um arquivo e salva
    pub fn update_and_save(&mut self, file_state: FileState) -> Result<()> {
        self.state.update_file(file_state);
        self.save()
    }

    /// Remove um arquivo e salva
    pub fn remove_and_save(&mut self, relative_path: &str) -> Result<()> {
        self.state.remove_file(relative_path);
        self.save()
    }

    /// Marca sincronização completa e salva
    pub fn mark_full_sync_and_save(&mut self) -> Result<()> {
        self.state.mark_full_sync();
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_creation() {
        let state = SyncState::new("/local/path".to_string(), "drive_id_123".to_string());
        assert_eq!(state.version, "1.0");
        assert_eq!(state.file_count(), 0);
        assert!(state.last_full_sync.is_none());
    }

    #[test]
    fn test_file_state_operations() {
        let mut state = SyncState::new("/local/path".to_string(), "drive_id_123".to_string());

        let file_state = FileState {
            relative_path: "test.txt".to_string(),
            drive_file_id: "file_123".to_string(),
            size: 1024,
            modified: 1234567890,
            md5_hash: "abc123".to_string(),
            last_synced: 1234567900,
        };

        // Adicionar arquivo
        state.update_file(file_state.clone());
        assert_eq!(state.file_count(), 1);
        assert!(state.is_synced("test.txt"));

        // Obter arquivo
        let retrieved = state.get_file("test.txt").unwrap();
        assert_eq!(retrieved.drive_file_id, "file_123");

        // Remover arquivo
        state.remove_file("test.txt");
        assert_eq!(state.file_count(), 0);
        assert!(!state.is_synced("test.txt"));
    }

    #[test]
    fn test_mark_full_sync() {
        let mut state = SyncState::new("/local/path".to_string(), "drive_id_123".to_string());
        assert!(state.last_full_sync.is_none());

        state.mark_full_sync();
        assert!(state.last_full_sync.is_some());
    }
}
