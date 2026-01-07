// Testes unitários para módulo de retry

use rustdrivesync::core::retry::{retry_with_backoff, RetryConfig};
use rustdrivesync::error::RustDriveSyncError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay_secs, 1);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.max_delay_secs, 60);
}

#[test]
fn test_retry_config_new() {
    let config = RetryConfig::new(5, 2);
    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.initial_delay_secs, 2);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.max_delay_secs, 60);
}

#[test]
fn test_retry_config_fields() {
    let config = RetryConfig::new(5, 2);

    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.initial_delay_secs, 2);
    assert_eq!(config.backoff_multiplier, 2.0); // Valor padrão
    assert_eq!(config.max_delay_secs, 60); // Valor padrão
}

#[tokio::test]
async fn test_retry_success_on_first_attempt() {
    let config = RetryConfig::new(3, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::Relaxed);
                Ok::<String, RustDriveSyncError>("sucesso".to_string())
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "sucesso");
    assert_eq!(counter.load(Ordering::Relaxed), 1); // Apenas 1 tentativa
}

#[tokio::test]
async fn test_retry_success_on_second_attempt() {
    let config = RetryConfig::new(3, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::Relaxed);
                if count == 0 {
                    Err(RustDriveSyncError::NetworkError {
                        message: "falha temporária".to_string(),
                    })
                } else {
                    Ok::<String, RustDriveSyncError>("sucesso".to_string())
                }
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "sucesso");
    assert_eq!(counter.load(Ordering::Relaxed), 2); // 2 tentativas
}

#[tokio::test]
async fn test_retry_all_attempts_fail() {
    let config = RetryConfig::new(3, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::Relaxed);
                Err::<String, RustDriveSyncError>(RustDriveSyncError::NetworkError {
                    message: "sempre falha".to_string(),
                })
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_err());
    assert_eq!(counter.load(Ordering::Relaxed), 3); // 3 tentativas
}

#[tokio::test]
async fn test_retry_config_invalid_error() {
    let config = RetryConfig::new(5, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::Relaxed);
                Err::<String, RustDriveSyncError>(RustDriveSyncError::ConfigInvalid {
                    message: "config inválida".to_string(),
                })
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_err());
    // retry_with_backoff sempre tenta max_attempts vezes, mesmo para erros de config
    assert_eq!(counter.load(Ordering::Relaxed), 5); // 5 tentativas
}

#[tokio::test]
async fn test_retry_rate_limit_error() {
    let config = RetryConfig::new(3, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::Relaxed);
                if count < 2 {
                    Err(RustDriveSyncError::RateLimitExceeded { retry_after: 1 })
                } else {
                    Ok::<String, RustDriveSyncError>("sucesso após rate limit".to_string())
                }
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 3); // 3 tentativas
}

#[tokio::test]
async fn test_retry_timeout_error() {
    let config = RetryConfig::new(2, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::Relaxed);
                if count == 0 {
                    Err(RustDriveSyncError::RequestTimeout)
                } else {
                    Ok::<String, RustDriveSyncError>("sucesso".to_string())
                }
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_retry_max_attempts_one() {
    let config = RetryConfig::new(1, 1);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        config,
        || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::Relaxed);
                Err::<String, RustDriveSyncError>(RustDriveSyncError::NetworkError {
                    message: "falha".to_string(),
                })
            }
        },
        "test_operation",
    )
    .await;

    assert!(result.is_err());
    assert_eq!(counter.load(Ordering::Relaxed), 1); // Apenas 1 tentativa
}

#[tokio::test]
async fn test_retry_with_different_return_types() {
    let config = RetryConfig::new(2, 1);

    // Teste com número
    let result_num = retry_with_backoff(
        config.clone(),
        || async { Ok::<i32, RustDriveSyncError>(42) },
        "test_num",
    )
    .await;
    assert_eq!(result_num.unwrap(), 42);

    // Teste com bool
    let result_bool = retry_with_backoff(
        config.clone(),
        || async { Ok::<bool, RustDriveSyncError>(true) },
        "test_bool",
    )
    .await;
    assert_eq!(result_bool.unwrap(), true);

    // Teste com struct complexo
    #[derive(Debug, PartialEq)]
    struct TestStruct {
        value: String,
    }

    let result_struct = retry_with_backoff(
        config,
        || async {
            Ok::<TestStruct, RustDriveSyncError>(TestStruct {
                value: "test".to_string(),
            })
        },
        "test_struct",
    )
    .await;
    assert_eq!(result_struct.unwrap().value, "test");
}
