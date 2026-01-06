use crate::error::{Result, RustDriveSyncError};
use notify_debouncer_mini::{
    new_debouncer, DebounceEventResult, DebouncedEvent, DebouncedEventKind, Debouncer,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tracing::{debug, error, info};

/// Tipo de evento de arquivo
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEventType {
    /// Arquivo criado
    Created,
    /// Arquivo modificado
    Modified,
    /// Arquivo deletado
    Deleted,
    /// Outro tipo de evento
    Other,
}

/// Evento de mudança de arquivo
#[derive(Debug, Clone)]
pub struct FileEvent {
    /// Tipo de evento
    pub event_type: FileEventType,
    /// Caminho do arquivo
    pub path: PathBuf,
}

impl FileEvent {
    /// Cria um novo evento de arquivo
    pub fn new(event_type: FileEventType, path: PathBuf) -> Self {
        Self { event_type, path }
    }

    /// Verifica se o evento é de criação ou modificação
    pub fn needs_sync(&self) -> bool {
        matches!(
            self.event_type,
            FileEventType::Created | FileEventType::Modified
        )
    }
}

/// Configuração do file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Tempo de debounce em segundos (padrão: 2s)
    pub debounce_duration: Duration,
    /// Padrões de arquivos a ignorar
    pub ignore_patterns: Vec<String>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_secs(2),
            ignore_patterns: vec![
                ".git".to_string(),
                ".gitignore".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                ".sync_state.json".to_string(),
                "state.json".to_string(),
            ],
        }
    }
}

impl WatcherConfig {
    /// Cria uma nova configuração com duração de debounce customizada
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    /// Adiciona padrões de arquivos a ignorar
    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    /// Verifica se um caminho deve ser ignorado
    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if path_str.contains(pattern.as_str()) {
                return true;
            }
        }

        false
    }
}

/// File watcher para monitoramento de mudanças
pub struct FileWatcher {
    /// Debouncer do notify
    _debouncer: Debouncer<notify::RecommendedWatcher>,
    /// Canal receptor de eventos
    event_receiver: Receiver<FileEvent>,
    /// Configuração
    config: WatcherConfig,
    /// Caminho sendo monitorado
    watched_path: PathBuf,
}

impl FileWatcher {
    /// Cria um novo file watcher
    pub fn new<P: AsRef<Path>>(path: P, config: WatcherConfig) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(RustDriveSyncError::FileNotFound {
                path: path.display().to_string(),
            });
        }

        if !path.is_dir() {
            return Err(RustDriveSyncError::DriveApiError {
                message: format!("Path não é um diretório: {}", path.display()),
            });
        }

        info!(
            "Inicializando file watcher para: {} (debounce: {:?})",
            path.display(),
            config.debounce_duration
        );

        // Canal para eventos processados
        let (event_tx, event_rx) = channel::<FileEvent>();

        // Clone para usar no closure
        let config_clone = config.clone();
        let event_tx_clone = event_tx.clone();

        // Criar debouncer
        let mut debouncer = new_debouncer(
            config.debounce_duration,
            move |result: DebounceEventResult| {
                Self::handle_debounced_events(result, &config_clone, &event_tx_clone);
            },
        )
        .map_err(|e| RustDriveSyncError::DriveApiError {
            message: format!("Erro ao criar debouncer: {}", e),
        })?;

        // Adicionar path ao watcher
        debouncer
            .watcher()
            .watch(path, notify::RecursiveMode::Recursive)
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao iniciar monitoramento: {}", e),
            })?;

        info!("File watcher iniciado com sucesso");

        Ok(Self {
            _debouncer: debouncer,
            event_receiver: event_rx,
            config,
            watched_path: path.to_path_buf(),
        })
    }

    /// Handler de eventos debounced
    fn handle_debounced_events(
        result: DebounceEventResult,
        config: &WatcherConfig,
        tx: &Sender<FileEvent>,
    ) {
        match result {
            Ok(events) => {
                for event in events {
                    Self::process_event(event, config, tx);
                }
            }
            Err(err) => {
                // err é um único notify::Error
                error!("Erro no file watcher: {:?}", err);
            }
        }
    }

    /// Processa um evento individual
    fn process_event(event: DebouncedEvent, config: &WatcherConfig, tx: &Sender<FileEvent>) {
        // Verificar se deve ignorar
        if config.should_ignore(&event.path) {
            debug!("Ignorando evento para: {}", event.path.display());
            return;
        }

        // Converter tipo de evento
        let event_type = match event.kind {
            DebouncedEventKind::Any => {
                // Para eventos 'Any', verificar se o arquivo existe
                if event.path.exists() {
                    FileEventType::Modified
                } else {
                    FileEventType::Deleted
                }
            }
            DebouncedEventKind::AnyContinuous => FileEventType::Modified,
            _ => FileEventType::Other, // Catch-all para outros tipos de eventos
        };

        let file_event = FileEvent::new(event_type.clone(), event.path.clone());

        debug!(
            "Evento detectado: {:?} - {}",
            event_type,
            event.path.display()
        );

        // Enviar evento
        if let Err(e) = tx.send(file_event) {
            error!("Erro ao enviar evento: {}", e);
        }
    }

    /// Aguarda o próximo evento
    pub fn next_event(&self) -> Result<FileEvent> {
        self.event_receiver
            .recv()
            .map_err(|e| RustDriveSyncError::DriveApiError {
                message: format!("Erro ao receber evento: {}", e),
            })
    }

    /// Tenta receber o próximo evento sem bloquear
    pub fn try_next_event(&self) -> Option<FileEvent> {
        self.event_receiver.try_recv().ok()
    }

    /// Retorna o caminho sendo monitorado
    pub fn watched_path(&self) -> &Path {
        &self.watched_path
    }

    /// Retorna a configuração
    pub fn config(&self) -> &WatcherConfig {
        &self.config
    }
}

/// Iterator para eventos do file watcher
pub struct FileWatcherIter<'a> {
    watcher: &'a FileWatcher,
}

impl<'a> FileWatcherIter<'a> {
    /// Cria um novo iterator
    pub fn new(watcher: &'a FileWatcher) -> Self {
        Self { watcher }
    }
}

impl<'a> Iterator for FileWatcherIter<'a> {
    type Item = FileEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.watcher.next_event().ok()
    }
}

impl FileWatcher {
    /// Retorna um iterator sobre os eventos
    pub fn events(&self) -> FileWatcherIter<'_> {
        FileWatcherIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert!(!config.ignore_patterns.is_empty());
        assert_eq!(config.debounce_duration, Duration::from_secs(2));
    }

    #[test]
    fn test_watcher_config_should_ignore() {
        let config = WatcherConfig::default();
        assert!(config.should_ignore(Path::new("/path/to/.git/config")));
        assert!(config.should_ignore(Path::new("/path/node_modules/pkg")));
        assert!(!config.should_ignore(Path::new("/path/to/file.txt")));
    }

    #[test]
    fn test_file_event_needs_sync() {
        let created = FileEvent::new(FileEventType::Created, PathBuf::from("test.txt"));
        assert!(created.needs_sync());

        let modified = FileEvent::new(FileEventType::Modified, PathBuf::from("test.txt"));
        assert!(modified.needs_sync());

        let deleted = FileEvent::new(FileEventType::Deleted, PathBuf::from("test.txt"));
        assert!(!deleted.needs_sync());
    }

    #[test]
    fn test_watcher_config_with_custom_debounce() {
        let config = WatcherConfig::default().with_debounce(Duration::from_secs(5));
        assert_eq!(config.debounce_duration, Duration::from_secs(5));
    }

    #[test]
    fn test_watcher_nonexistent_path() {
        let config = WatcherConfig::default();
        let result = FileWatcher::new("/nonexistent/path", config);
        assert!(result.is_err());
    }
}
