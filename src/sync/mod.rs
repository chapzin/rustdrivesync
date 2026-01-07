// Módulo de sincronização de arquivos

pub mod engine;
pub mod engine_v2;
pub mod scanner;
pub mod state;
pub mod tracker;

// Re-exports públicos (mantém compatibilidade mas usa nova versão)
pub use engine_v2::{SyncEngine, SyncMode, SyncResult, SyncStats};
pub use scanner::{FileScanner, LocalFile};
pub use state::{SyncState, SyncStateManager};
pub use tracker::ChangeTracker;
