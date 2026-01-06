use std::path::PathBuf;

/// Scanner de arquivos que varre diretórios
pub struct FileScanner {
    pub path: PathBuf,
}

impl FileScanner {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // TODO: Implementar scan de arquivos
}
