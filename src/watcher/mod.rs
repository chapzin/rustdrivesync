// Módulo de monitoramento de arquivos
pub mod file_watcher;

// Re-exports públicos
pub use file_watcher::{FileEvent, FileEventType, FileWatcher, WatcherConfig};
