use crate::error::{Result, RustDriveSyncError};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Representa um arquivo local encontrado durante o scan
#[derive(Debug, Clone)]
pub struct LocalFile {
    /// Caminho absoluto do arquivo
    pub path: PathBuf,
    /// Caminho relativo a partir do diretório raiz de sincronização
    pub relative_path: PathBuf,
    /// Tamanho do arquivo em bytes
    pub size: u64,
    /// Timestamp da última modificação (Unix timestamp)
    pub modified: i64,
    /// Hash MD5 do arquivo (calculado sob demanda)
    pub md5_hash: Option<String>,
}

impl LocalFile {
    /// Cria um novo LocalFile a partir de um caminho absoluto e base
    pub fn from_path(path: PathBuf, base_path: &Path) -> Result<Self> {
        let metadata = fs::metadata(&path).map_err(|e| RustDriveSyncError::FileReadError {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;

        let relative_path = path
            .strip_prefix(base_path)
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao calcular caminho relativo: {}", e),
            })?
            .to_path_buf();

        let modified = metadata
            .modified()
            .map_err(|e| RustDriveSyncError::FileReadError {
                path: path.display().to_string(),
                message: e.to_string(),
            })?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| RustDriveSyncError::FileReadError {
                path: path.display().to_string(),
                message: e.to_string(),
            })?
            .as_secs() as i64;

        Ok(Self {
            path,
            relative_path,
            size: metadata.len(),
            modified,
            md5_hash: None,
        })
    }

    /// Calcula o hash MD5 do arquivo
    pub fn calculate_md5(&mut self) -> Result<String> {
        use md5::{Digest, Md5};

        let content = fs::read(&self.path).map_err(|e| RustDriveSyncError::FileReadError {
            path: self.path.display().to_string(),
            message: e.to_string(),
        })?;

        let mut hasher = Md5::new();
        hasher.update(&content);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);

        self.md5_hash = Some(hash.clone());
        Ok(hash)
    }
}

/// Scanner de arquivos locais
pub struct FileScanner {
    /// Padrões de arquivos a ignorar (glob patterns)
    ignore_patterns: Vec<String>,
    /// Tamanho máximo de arquivo em bytes (None = sem limite)
    max_file_size: Option<u64>,
}

impl FileScanner {
    /// Cria um novo scanner com configurações padrão
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![
                ".git".to_string(),
                ".gitignore".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            max_file_size: None,
        }
    }

    /// Configura os padrões de arquivos a ignorar
    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    /// Configura o tamanho máximo de arquivo
    pub fn with_max_file_size(mut self, max_size: u64) -> Self {
        self.max_file_size = Some(max_size);
        self
    }

    /// Escaneia um diretório e retorna todos os arquivos encontrados
    pub fn scan_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<LocalFile>> {
        let dir_path = dir_path.as_ref();

        if !dir_path.exists() {
            return Err(RustDriveSyncError::FileNotFound {
                path: dir_path.display().to_string(),
            });
        }

        if !dir_path.is_dir() {
            return Err(RustDriveSyncError::DriveApiError {
                message: format!("Path não é um diretório: {}", dir_path.display()),
            });
        }

        info!("Escaneando diretório: {}", dir_path.display());

        let mut files = Vec::new();
        let walker = WalkDir::new(dir_path).follow_links(false);

        for entry in walker {
            let entry = entry.map_err(|e| RustDriveSyncError::FileReadError {
                path: dir_path.display().to_string(),
                message: e.to_string(),
            })?;

            // Pular diretórios
            if entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();

            // Verificar se deve ser ignorado
            if self.should_ignore(path) {
                debug!("Ignorando arquivo: {}", path.display());
                continue;
            }

            // Criar LocalFile
            let local_file = LocalFile::from_path(path.to_path_buf(), dir_path)?;

            // Verificar tamanho máximo
            if let Some(max_size) = self.max_file_size {
                if local_file.size > max_size {
                    debug!(
                        "Arquivo muito grande ({}), ignorando: {}",
                        local_file.size,
                        path.display()
                    );
                    continue;
                }
            }

            files.push(local_file);
        }

        info!("Encontrados {} arquivos", files.len());
        Ok(files)
    }

    /// Verifica se um arquivo deve ser ignorado baseado nos padrões
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if path_str.contains(pattern.as_str()) {
                return true;
            }
        }

        false
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuração do scanner (para acesso externo)
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub max_file_size: Option<u64>,
}

impl FileScanner {
    /// Retorna a configuração do scanner
    pub fn config(&self) -> ScannerConfig {
        ScannerConfig {
            max_file_size: self.max_file_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = FileScanner::new();
        assert!(!scanner.ignore_patterns.is_empty());
        assert!(scanner.max_file_size.is_none());
    }

    #[test]
    fn test_should_ignore() {
        let scanner = FileScanner::new();
        assert!(scanner.should_ignore(Path::new("/path/to/.git/config")));
        assert!(scanner.should_ignore(Path::new("/path/node_modules/package")));
        assert!(!scanner.should_ignore(Path::new("/path/to/file.txt")));
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let scanner = FileScanner::new();
        let result = scanner.scan_directory("/nonexistent/directory");
        assert!(result.is_err());
    }
}
