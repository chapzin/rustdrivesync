use super::schema::Config;
use crate::error::{Result, RustDriveSyncError};

/// Valida a configuração carregada
pub fn validate_config(config: &Config) -> Result<()> {
    // Validar pasta de origem existe
    if !config.source.path.exists() {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: format!(
                "Pasta de origem não existe: {}",
                config.source.path.display()
            ),
        });
    }

    // Validar que tem pelo menos um identificador de pasta do Drive
    if config.google_drive.target_folder_id.is_none()
        && config.google_drive.target_folder_name.is_none()
    {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: "É necessário especificar target_folder_id ou target_folder_name".to_string(),
        });
    }

    // Validar modo de sincronização
    if config.sync.mode != "once" && config.sync.mode != "watch" {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: format!(
                "Modo de sync inválido: {}. Use 'once' ou 'watch'",
                config.sync.mode
            ),
        });
    }

    // Validar resolução de conflitos
    if !["overwrite", "skip", "rename"].contains(&config.sync.conflict_resolution.as_str()) {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: format!(
                "Resolução de conflito inválida: {}. Use 'overwrite', 'skip' ou 'rename'",
                config.sync.conflict_resolution
            ),
        });
    }

    // Validar tamanhos
    if config.sync.chunk_size_mb == 0 {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: "chunk_size_mb deve ser maior que 0".to_string(),
        });
    }

    if config.sync.max_file_size_mb == 0 {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: "max_file_size_mb deve ser maior que 0".to_string(),
        });
    }

    // Validar retry config
    if config.retry.max_attempts == 0 {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: "max_attempts deve ser maior que 0".to_string(),
        });
    }

    if config.retry.backoff_multiplier <= 0.0 {
        return Err(RustDriveSyncError::ConfigInvalid {
            message: "backoff_multiplier deve ser maior que 0".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_missing_drive_folder() {
        let config = Config {
            general: GeneralConfig {
                log_level: "info".to_string(),
                log_file: None,
                log_format: "text".to_string(),
            },
            source: SourceConfig {
                path: PathBuf::from("."),
                recursive: true,
                ignore_hidden: true,
                follow_symlinks: false,
                ignore_patterns: vec![],
            },
            google_drive: GoogleDriveConfig {
                credentials_file: PathBuf::from("credentials.json"),
                token_file: PathBuf::from("token.json"),
                target_folder_id: None,
                target_folder_name: None,
                scopes: vec![],
            },
            sync: SyncConfig {
                mode: "once".to_string(),
                interval_seconds: 300,
                conflict_resolution: "overwrite".to_string(),
                max_file_size_mb: 100,
                chunk_size_mb: 5,
                max_concurrent_uploads: 4,
                verify_upload: true,
                preserve_folder_structure: true,
                delete_remote_on_local_delete: false,
            },
            retry: RetryConfig::default(),
            notifications: NotificationsConfig::default(),
            state: StateConfig::default(),
        };

        let result = validate_config(&config);
        assert!(result.is_err());
    }
}
