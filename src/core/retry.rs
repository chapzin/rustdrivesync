// Retry com exponential backoff

use crate::error::Result;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuração de retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Número máximo de tentativas
    pub max_attempts: u32,
    /// Delay inicial em segundos
    pub initial_delay_secs: u64,
    /// Multiplicador para backoff exponencial
    pub backoff_multiplier: f64,
    /// Delay máximo em segundos
    pub max_delay_secs: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_secs: 1,
            backoff_multiplier: 2.0,
            max_delay_secs: 60,
        }
    }
}

impl RetryConfig {
    /// Cria configuração com valores personalizados
    pub fn new(max_attempts: u32, initial_delay_secs: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_secs,
            ..Default::default()
        }
    }

    /// Calcula o delay para uma tentativa específica
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_secs = (self.initial_delay_secs as f64)
            * self.backoff_multiplier.powi(attempt as i32 - 1);

        let delay_secs = delay_secs.min(self.max_delay_secs as f64);

        Duration::from_secs_f64(delay_secs)
    }
}

/// Executa uma operação com retry e exponential backoff
///
/// # Argumentos
/// * `config` - Configuração de retry
/// * `operation` - Função assíncrona a ser executada
/// * `operation_name` - Nome da operação (para logs)
///
/// # Exemplo
/// ```no_run
/// use rustdrivesync::core::retry::{retry_with_backoff, RetryConfig};
/// use rustdrivesync::error::Result;
///
/// # async fn example() -> Result<()> {
/// let config = RetryConfig::default();
/// let result = retry_with_backoff(
///     config,
///     || async { do_network_request().await },
///     "network_request"
/// ).await?;
/// # Ok(())
/// # }
/// # async fn do_network_request() -> Result<String> { Ok("".into()) }
/// ```
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    mut operation: F,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;

    loop {
        attempt += 1;

        debug!(
            "Tentativa {}/{} para operação: {}",
            attempt, config.max_attempts, operation_name
        );

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "Operação '{}' bem-sucedida após {} tentativas",
                        operation_name, attempt
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= config.max_attempts {
                    warn!(
                        "Operação '{}' falhou após {} tentativas: {}",
                        operation_name, attempt, e
                    );
                    return Err(e);
                }

                let delay = config.calculate_delay(attempt);
                warn!(
                    "Tentativa {}/{} falhou para '{}': {}. Aguardando {:?} antes de retry...",
                    attempt, config.max_attempts, operation_name, e, delay
                );

                sleep(delay).await;
            }
        }
    }
}

/// Verifica se um erro é retryable (temporário)
///
/// Erros de rede, timeouts e rate limits devem ser retryados.
/// Erros de autenticação, permissão ou validação não devem.
pub fn is_retryable_error(error: &crate::error::RustDriveSyncError) -> bool {
    use crate::error::RustDriveSyncError;

    matches!(
        error,
        RustDriveSyncError::NetworkError { .. }
            | RustDriveSyncError::RequestTimeout
            | RustDriveSyncError::RateLimitExceeded { .. }
            | RustDriveSyncError::DriveApiError { .. } // Alguns erros de API podem ser temporários
    )
}

/// Wrapper para retry condicional (apenas se o erro for retryable)
pub async fn retry_if_retryable<F, Fut, T>(
    config: RetryConfig,
    operation: F,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    retry_with_backoff(config, operation, operation_name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_secs, 1);
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig::default();

        // Tentativa 1: 1s * 2^0 = 1s
        assert_eq!(config.calculate_delay(1), Duration::from_secs(1));

        // Tentativa 2: 1s * 2^1 = 2s
        assert_eq!(config.calculate_delay(2), Duration::from_secs(2));

        // Tentativa 3: 1s * 2^2 = 4s
        assert_eq!(config.calculate_delay(3), Duration::from_secs(4));

        // Tentativa 10 seria 512s, mas max é 60s
        assert_eq!(config.calculate_delay(10), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::new(3, 1);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            config,
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<i32, crate::error::RustDriveSyncError>(42)
                }
            },
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Apenas 1 tentativa
    }

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let config = RetryConfig::new(3, 1);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            config,
            || {
                let c = counter_clone.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Err(crate::error::RustDriveSyncError::NetworkError {
                            message: "Temporary failure".to_string(),
                        })
                    } else {
                        Ok::<i32, crate::error::RustDriveSyncError>(42)
                    }
                }
            },
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2); // 2 tentativas
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let config = RetryConfig::new(3, 1);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            config,
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, crate::error::RustDriveSyncError>(
                        crate::error::RustDriveSyncError::NetworkError {
                            message: "Persistent failure".to_string(),
                        },
                    )
                }
            },
            "test_operation",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Todas as 3 tentativas
    }
}
