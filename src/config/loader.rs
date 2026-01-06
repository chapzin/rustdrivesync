use super::schema::Config;
use crate::error::{Result, RustDriveSyncError};
use std::path::Path;

/// Carrega a configuração a partir de um arquivo TOML
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path_ref = path.as_ref();

    if !path_ref.exists() {
        return Err(RustDriveSyncError::ConfigNotFound {
            path: path_ref.display().to_string(),
        });
    }

    let content =
        std::fs::read_to_string(path_ref).map_err(|e| RustDriveSyncError::ConfigInvalid {
            message: format!("Erro ao ler arquivo: {}", e),
        })?;

    let config: Config =
        toml::from_str(&content).map_err(|e| RustDriveSyncError::ConfigInvalid {
            message: format!("Erro ao parsear TOML: {}", e),
        })?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_config() {
        let result = load_config("/nonexistent/config.toml");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RustDriveSyncError::ConfigNotFound { .. }
        ));
    }
}
