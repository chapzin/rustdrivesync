// Módulos de integração com Google Drive
pub mod auth;
pub mod client;
pub mod models;
pub mod rate_limiter;
pub mod storage_impl;
pub mod upload;

// Re-exports
pub use rate_limiter::DriveRateLimiter;
pub use storage_impl::DriveStorageBackend;
