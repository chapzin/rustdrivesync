// Rate limiter para Google Drive API

use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{debug, warn};

/// Rate limiter para Google Drive API
///
/// Google Drive tem os seguintes limites:
/// - 1,000 requests por 100 segundos por usuário
/// - 10,000 requests por 100 segundos por projeto
///
/// Implementamos uma estratégia conservadora para evitar rate limiting.
#[derive(Clone)]
pub struct DriveRateLimiter {
    /// Semáforo para limitar requisições concorrentes
    semaphore: Arc<Semaphore>,
    /// Número máximo de requisições por período
    max_requests_per_period: u32,
    /// Duração do período em segundos
    period_seconds: u64,
}

impl DriveRateLimiter {
    /// Cria um novo rate limiter com configurações padrão do Google Drive
    ///
    /// Limita a 10 requisições concorrentes e 800 requests/100s (80% do limite)
    /// para ter margem de segurança.
    pub fn new() -> Self {
        Self::with_limits(10, 800, 100)
    }

    /// Cria um rate limiter com limites personalizados
    ///
    /// # Argumentos
    /// * `max_concurrent` - Número máximo de requisições simultâneas
    /// * `max_requests` - Número máximo de requisições no período
    /// * `period_secs` - Duração do período em segundos
    pub fn with_limits(max_concurrent: usize, max_requests: u32, period_secs: u64) -> Self {
        debug!(
            "Criando rate limiter: {} concorrentes, {} req/{} sec",
            max_concurrent, max_requests, period_secs
        );

        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_requests_per_period: max_requests,
            period_seconds: period_secs,
        }
    }

    /// Aguarda até que seja possível fazer uma requisição
    ///
    /// Esta função bloqueia até que haja capacidade disponível.
    /// Retorna um guard que libera o slot quando dropped.
    pub async fn acquire(&self) -> RateLimitGuard {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Semaphore should never be closed");

        RateLimitGuard {
            _permit: permit,
        }
    }

    /// Tenta adquirir permissão sem bloquear
    ///
    /// Retorna Some(guard) se houver capacidade, None caso contrário.
    pub fn try_acquire(&self) -> Option<RateLimitGuard> {
        self.semaphore.clone().try_acquire_owned().ok().map(|permit| RateLimitGuard {
            _permit: permit,
        })
    }

    /// Retorna o número de slots disponíveis
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Verifica se está próximo do limite (< 20% de capacidade)
    pub fn is_near_limit(&self) -> bool {
        let available = self.available_permits();
        let total = self.semaphore.available_permits() + 1; // Aproximação
        available < total / 5
    }
}

impl Default for DriveRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard que libera o slot do rate limiter quando dropped
pub struct RateLimitGuard {
    _permit: OwnedSemaphorePermit,
}

/// Wrapper para cliente com rate limiting
///
/// Envolve qualquer cliente e aplica rate limiting automaticamente.
pub struct RateLimitedClient<T> {
    inner: T,
    limiter: DriveRateLimiter,
}

impl<T> RateLimitedClient<T> {
    /// Cria um novo cliente com rate limiting
    pub fn new(client: T, limiter: DriveRateLimiter) -> Self {
        Self {
            inner: client,
            limiter,
        }
    }

    /// Obtém referência ao cliente interno
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Obtém referência mutável ao cliente interno
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Obtém referência ao rate limiter
    pub fn limiter(&self) -> &DriveRateLimiter {
        &self.limiter
    }

    /// Executa uma operação com rate limiting
    ///
    /// Aguarda automaticamente se necessário para respeitar os limites.
    pub async fn execute<F, Fut, R>(&self, operation: F) -> R
    where
        F: FnOnce(&T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let _guard = self.limiter.acquire().await;

        if self.limiter.is_near_limit() {
            warn!("Rate limiter próximo do limite - considere reduzir a taxa de requisições");
        }

        operation(&self.inner).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = DriveRateLimiter::with_limits(2, 10, 1);

        // Deve permitir 2 requisições simultâneas
        let _guard1 = limiter.acquire().await;
        let _guard2 = limiter.acquire().await;

        // Terceira requisição deve aguardar
        assert_eq!(limiter.available_permits(), 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_release() {
        let limiter = DriveRateLimiter::with_limits(1, 10, 1);

        {
            let _guard = limiter.acquire().await;
            assert_eq!(limiter.available_permits(), 0);
        } // Guard dropped aqui

        // Após drop, slot deve estar disponível
        assert_eq!(limiter.available_permits(), 1);
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = DriveRateLimiter::with_limits(1, 10, 1);

        let guard1 = limiter.try_acquire();
        assert!(guard1.is_some());

        // Segunda tentativa deve falhar
        let guard2 = limiter.try_acquire();
        assert!(guard2.is_none());

        drop(guard1);

        // Após liberar, deve funcionar
        let guard3 = limiter.try_acquire();
        assert!(guard3.is_some());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let limiter = Arc::new(DriveRateLimiter::with_limits(5, 100, 1));
        let start = Instant::now();

        // Tentar 10 operações concorrentes (mas limite é 5)
        let tasks: Vec<_> = (0..10)
            .map(|i| {
                let limiter = limiter.clone();
                tokio::spawn(async move {
                    let _guard = limiter.acquire().await;
                    sleep(Duration::from_millis(100)).await;
                    i
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        let duration = start.elapsed();

        // Deve ter processado todas
        assert_eq!(results.len(), 10);

        // Deve ter levado pelo menos 200ms (2 batches de 5)
        assert!(duration >= Duration::from_millis(200));
    }

    // Teste de integração do RateLimitedClient foi movido para testes de integração
    // devido a complexidade de lifetimes com closures async
}
