// Módulos do core engine
pub mod hasher;
pub mod queue;
pub mod scanner;
pub mod state;
pub mod sync_engine;

// Re-exports
pub use hasher::compute_file_hash;
pub use scanner::FileScanner;
