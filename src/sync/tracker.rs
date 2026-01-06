use crate::sync::scanner::LocalFile;
use crate::sync::state::{FileState, SyncState};
use std::collections::HashSet;
use tracing::{debug, info};

/// Tipo de mudança detectada
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Arquivo novo que nunca foi sincronizado
    New,
    /// Arquivo modificado (tamanho ou timestamp diferente)
    Modified,
    /// Arquivo deletado localmente
    Deleted,
    /// Arquivo não modificado
    Unchanged,
}

/// Representa uma mudança detectada
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Caminho relativo do arquivo
    pub relative_path: String,
    /// Tipo de mudança
    pub change_type: ChangeType,
    /// Arquivo local (None para arquivos deletados)
    pub local_file: Option<LocalFile>,
    /// Estado anterior (None para arquivos novos)
    pub previous_state: Option<FileState>,
}

impl FileChange {
    /// Cria uma mudança para arquivo novo
    pub fn new(local_file: LocalFile) -> Self {
        Self {
            relative_path: local_file.relative_path.display().to_string(),
            change_type: ChangeType::New,
            local_file: Some(local_file),
            previous_state: None,
        }
    }

    /// Cria uma mudança para arquivo modificado
    pub fn modified(local_file: LocalFile, previous_state: FileState) -> Self {
        Self {
            relative_path: local_file.relative_path.display().to_string(),
            change_type: ChangeType::Modified,
            local_file: Some(local_file),
            previous_state: Some(previous_state),
        }
    }

    /// Cria uma mudança para arquivo deletado
    pub fn deleted(relative_path: String, previous_state: FileState) -> Self {
        Self {
            relative_path,
            change_type: ChangeType::Deleted,
            local_file: None,
            previous_state: Some(previous_state),
        }
    }

    /// Cria uma mudança para arquivo não modificado
    pub fn unchanged(local_file: LocalFile, previous_state: FileState) -> Self {
        Self {
            relative_path: local_file.relative_path.display().to_string(),
            change_type: ChangeType::Unchanged,
            local_file: Some(local_file),
            previous_state: Some(previous_state),
        }
    }
}

/// Rastreador de mudanças
pub struct ChangeTracker;

impl ChangeTracker {
    /// Detecta mudanças entre o estado atual e os arquivos locais
    pub fn detect_changes(
        local_files: &[LocalFile],
        sync_state: &SyncState,
    ) -> Vec<FileChange> {
        let mut changes = Vec::new();

        // Criar conjunto de caminhos locais para facilitar busca
        let local_paths: HashSet<String> = local_files
            .iter()
            .map(|f| f.relative_path.display().to_string())
            .collect();

        // Detectar arquivos novos e modificados
        for local_file in local_files {
            let relative_path = local_file.relative_path.display().to_string();

            match sync_state.get_file(&relative_path) {
                Some(previous_state) => {
                    // Arquivo existe no estado - verificar se foi modificado
                    if Self::is_modified(local_file, previous_state) {
                        debug!("Arquivo modificado: {}", relative_path);
                        changes.push(FileChange::modified(
                            local_file.clone(),
                            previous_state.clone(),
                        ));
                    } else {
                        debug!("Arquivo não modificado: {}", relative_path);
                        changes.push(FileChange::unchanged(
                            local_file.clone(),
                            previous_state.clone(),
                        ));
                    }
                }
                None => {
                    // Arquivo novo
                    debug!("Arquivo novo: {}", relative_path);
                    changes.push(FileChange::new(local_file.clone()));
                }
            }
        }

        // Detectar arquivos deletados
        for (relative_path, previous_state) in &sync_state.files {
            if !local_paths.contains(relative_path) {
                debug!("Arquivo deletado: {}", relative_path);
                changes.push(FileChange::deleted(
                    relative_path.clone(),
                    previous_state.clone(),
                ));
            }
        }

        info!(
            "Mudanças detectadas: {} novos, {} modificados, {} deletados, {} não modificados",
            changes
                .iter()
                .filter(|c| c.change_type == ChangeType::New)
                .count(),
            changes
                .iter()
                .filter(|c| c.change_type == ChangeType::Modified)
                .count(),
            changes
                .iter()
                .filter(|c| c.change_type == ChangeType::Deleted)
                .count(),
            changes
                .iter()
                .filter(|c| c.change_type == ChangeType::Unchanged)
                .count(),
        );

        changes
    }

    /// Verifica se um arquivo foi modificado
    fn is_modified(local_file: &LocalFile, previous_state: &FileState) -> bool {
        // Comparar tamanho
        if local_file.size != previous_state.size {
            return true;
        }

        // Comparar timestamp de modificação
        if local_file.modified != previous_state.modified {
            return true;
        }

        // Se tiver MD5, comparar
        if let Some(ref local_md5) = local_file.md5_hash {
            if local_md5 != &previous_state.md5_hash {
                return true;
            }
        }

        false
    }

    /// Filtra mudanças para retornar apenas as que precisam ser sincronizadas
    pub fn filter_sync_needed(changes: Vec<FileChange>) -> Vec<FileChange> {
        changes
            .into_iter()
            .filter(|change| {
                matches!(
                    change.change_type,
                    ChangeType::New | ChangeType::Modified
                )
            })
            .collect()
    }

    /// Agrupa mudanças por tipo
    pub fn group_by_type(changes: &[FileChange]) -> (Vec<&FileChange>, Vec<&FileChange>, Vec<&FileChange>) {
        let new: Vec<&FileChange> = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::New)
            .collect();

        let modified: Vec<&FileChange> = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .collect();

        let deleted: Vec<&FileChange> = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Deleted)
            .collect();

        (new, modified, deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_local_file(path: &str, size: u64, modified: i64) -> LocalFile {
        LocalFile {
            path: PathBuf::from(path),
            relative_path: PathBuf::from(path),
            size,
            modified,
            md5_hash: None,
        }
    }

    fn create_test_file_state(
        path: &str,
        size: u64,
        modified: i64,
        md5: &str,
    ) -> FileState {
        FileState {
            relative_path: path.to_string(),
            drive_file_id: "test_id".to_string(),
            size,
            modified,
            md5_hash: md5.to_string(),
            last_synced: modified,
        }
    }

    #[test]
    fn test_detect_new_file() {
        let local_files = vec![create_test_local_file("new.txt", 100, 1000)];
        let sync_state = SyncState::new("/local".to_string(), "drive_id".to_string());

        let changes = ChangeTracker::detect_changes(&local_files, &sync_state);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::New);
    }

    #[test]
    fn test_detect_modified_file() {
        let local_files = vec![create_test_local_file("modified.txt", 200, 2000)];
        let mut sync_state = SyncState::new("/local".to_string(), "drive_id".to_string());
        sync_state.update_file(create_test_file_state("modified.txt", 100, 1000, "old_hash"));

        let changes = ChangeTracker::detect_changes(&local_files, &sync_state);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Modified);
    }

    #[test]
    fn test_detect_deleted_file() {
        let local_files = vec![];
        let mut sync_state = SyncState::new("/local".to_string(), "drive_id".to_string());
        sync_state.update_file(create_test_file_state("deleted.txt", 100, 1000, "hash"));

        let changes = ChangeTracker::detect_changes(&local_files, &sync_state);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Deleted);
    }

    #[test]
    fn test_filter_sync_needed() {
        let changes = vec![
            FileChange::new(create_test_local_file("new.txt", 100, 1000)),
            FileChange::deleted(
                "deleted.txt".to_string(),
                create_test_file_state("deleted.txt", 100, 1000, "hash"),
            ),
        ];

        let sync_needed = ChangeTracker::filter_sync_needed(changes);
        assert_eq!(sync_needed.len(), 1);
        assert_eq!(sync_needed[0].change_type, ChangeType::New);
    }
}
