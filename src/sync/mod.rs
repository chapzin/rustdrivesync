// Módulo de sincronização de arquivos

pub mod engine;
pub mod scanner;
pub mod state;
pub mod tracker;

// Re-exports públicos
pub use engine::{SyncEngine, SyncMode, SyncResult, SyncStats};
pub use scanner::{FileScanner, LocalFile};
pub use state::{SyncState, SyncStateManager};
pub use tracker::ChangeTracker;
