// Biblioteca principal do RustDriveSync

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod google_drive;
pub mod logging;
pub mod sync;
pub mod watcher;

// Re-exports públicos
pub use config::Config;
pub use error::{Result, RustDriveSyncError};
