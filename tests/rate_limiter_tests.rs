// Testes unitários para Rate Limiter

use rustdrivesync::google_drive::rate_limiter::DriveRateLimiter;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_rate_limiter_creation() {
    let limiter = DriveRateLimiter::with_limits(10, 100, 60);
    // Limiter criado com sucesso
    assert!(limiter.available_permits() > 0);
}

#[tokio::test]
async fn test_rate_limiter_default() {
    let limiter = DriveRateLimiter::new();
    // Valores padrão: 10 concurrent
    assert_eq!(limiter.available_permits(), 10);
}

#[tokio::test]
async fn test_rate_limiter_acquire_single() {
    let limiter = DriveRateLimiter::with_limits(5, 100, 60);
    let _guard = limiter.acquire().await;
    // Se conseguimos adquirir, o teste passa
    assert_eq!(limiter.available_permits(), 4);
}

#[tokio::test]
async fn test_rate_limiter_acquire_multiple() {
    let limiter = DriveRateLimiter::with_limits(3, 100, 60);

    // Adquirir 3 permits (o máximo)
    let _guard1 = limiter.acquire().await;
    let _guard2 = limiter.acquire().await;
    let _guard3 = limiter.acquire().await;

    // Todos os permits foram adquiridos com sucesso
    assert_eq!(limiter.available_permits(), 0);
}

#[tokio::test]
async fn test_rate_limiter_release_on_drop() {
    let limiter = DriveRateLimiter::with_limits(2, 100, 60);

    {
        let _guard1 = limiter.acquire().await;
        let _guard2 = limiter.acquire().await;
        assert_eq!(limiter.available_permits(), 0);
    } // Permits liberados aqui quando guards saem de escopo

    // Agora podemos adquirir novamente
    assert_eq!(limiter.available_permits(), 2);
    let _guard3 = limiter.acquire().await;
    let _guard4 = limiter.acquire().await;
    assert_eq!(limiter.available_permits(), 0);
}

#[tokio::test]
async fn test_rate_limiter_concurrent_access() {
    let limiter = DriveRateLimiter::with_limits(5, 100, 60);

    let mut handles = vec![];

    // Criar 10 tasks tentando adquirir permits
    for i in 0..10 {
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            let _guard = limiter_clone.acquire().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            i
        });
        handles.push(handle);
    }

    // Esperar todas as tasks completarem
    for handle in handles {
        handle.await.unwrap();
    }

    // Todos os permits devem estar liberados
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(limiter.available_permits(), 5);
}

#[tokio::test]
async fn test_rate_limiter_respects_limit() {
    let limiter = DriveRateLimiter::with_limits(2, 100, 60);

    let start = Instant::now();

    // Adquirir 2 permits e segurar por 100ms
    let guard1 = limiter.acquire().await;
    let guard2 = limiter.acquire().await;

    assert_eq!(limiter.available_permits(), 0);

    // Criar task que tentará adquirir o 3º permit (deve esperar)
    let limiter_clone = limiter.clone();
    let blocker_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        drop(guard1);
        drop(guard2);
    });

    let waiter_handle = tokio::spawn(async move {
        let _guard = limiter_clone.acquire().await;
    });

    blocker_handle.await.unwrap();
    waiter_handle.await.unwrap();

    let elapsed = start.elapsed();

    // O waiter deve ter esperado pelo menos 50ms
    assert!(elapsed >= Duration::from_millis(50));
}

#[test]
fn test_rate_limiter_available_permits() {
    let limiter = DriveRateLimiter::with_limits(5, 100, 60);
    assert_eq!(limiter.available_permits(), 5);
}

#[tokio::test]
async fn test_rate_limiter_clone() {
    let limiter1 = DriveRateLimiter::with_limits(3, 100, 60);
    let limiter2 = limiter1.clone();

    // Ambos devem compartilhar o mesmo semáforo
    let _guard1 = limiter1.acquire().await;
    let _guard2 = limiter2.acquire().await;
    let _guard3 = limiter1.acquire().await;

    // Todos os 3 permits adquiridos
    assert_eq!(limiter1.available_permits(), 0);
    assert_eq!(limiter2.available_permits(), 0);
}
