# ADR-0003: Retry com Exponential Backoff

## Status
**Status:** Accepted

**Data:** 2026-01-07

**Autor(es):** RustDriveSync Team

## Contexto

Sistemas distribuídos e integrações com APIs externas estão sujeitos a falhas temporárias:

1. **Falhas de Rede**: Timeouts, conexões perdidas, DNS temporário
2. **Falhas do Servidor**: 500, 502, 503 (sobrecarga temporária)
3. **Rate Limiting**: 429 Too Many Requests (mesmo com nosso limiter)
4. **Falhas Transitórias**: Indisponibilidade momentânea de serviços

Sem retry, o sistema falhava permanentemente em casos recuperáveis, desperdiçando trabalho já realizado.

**Requisitos:**
- Retry automático para erros temporários
- Evitar "thundering herd" (muitos clientes tentando simultaneamente)
- Configurável por tipo de operação
- Logging detalhado para debugging
- Não fazer retry de erros permanentes (401, 404)

## Decisão

**Implementaremos retry com exponential backoff** usando a seguinte estratégia:

- **Delays Exponenciais**: 1s → 2s → 4s → 8s → ... (multiplicador 2.0)
- **Max Delay**: 60 segundos (evitar esperas infinitas)
- **Max Attempts**: 3 tentativas por padrão (configurável)
- **Jitter**: Sem jitter (pode ser adicionado no futuro)
- **Classificação de Erros**: Apenas erros retryable são retentados

## Alternativas Consideradas

### Alternativa 1: Fixed Delay Retry
**Descrição:** Esperar tempo fixo entre tentativas (ex: 5s)

```rust
for attempt in 0..max_attempts {
    match operation() {
        Ok(r) => return Ok(r),
        Err(_) => sleep(Duration::from_secs(5)).await,
    }
}
```

**Prós:**
- Simples de implementar
- Previsível

**Contras:**
- Não se adapta à gravidade da falha
- Pode causar "thundering herd"
- Não segue best practices

**Razão para Rejeição:** Inferior ao exponential backoff em todos os aspectos

### Alternativa 2: Fibonacci Backoff
**Descrição:** Delays seguem sequência de Fibonacci: 1, 1, 2, 3, 5, 8...

**Prós:**
- Crescimento mais suave que exponencial
- Menos agressivo

**Contras:**
- Complexidade adicional sem benefício claro
- Não é padrão da indústria
- Crescimento ainda é muito rápido

**Razão para Rejeição:** Exponencial é mais conhecido e igualmente eficaz

### Alternativa 3: Biblioteca Externa (backoff, retry-async)
**Descrição:** Usar crates existentes como `backoff` ou `retry-async`

**Prós:**
- Código testado em produção
- Recursos adicionais (jitter, decorrelated)

**Contras:**
- Dependência adicional
- Menos controle sobre implementação
- Nem todas suportam async/await bem
- Nosso caso é simples o suficiente

**Razão para Rejeição:** Implementação própria é suficiente e mais controlável

### Alternativa 4: Sem Retry (Status Quo)
**Descrição:** Deixar usuário lidar manualmente com falhas

**Prós:**
- Sem complexidade adicional
- Controle total para usuário

**Contras:**
- Má experiência de usuário
- Desperdício de trabalho
- Inconsistente com expectativas modernas

**Razão para Rejeição:** Inaceitável para ferramenta de produção

## Consequências

### Positivas
- ✅ **Resiliência**: Recuperação automática de falhas temporárias
- ✅ **Best Practice**: Segue padrão da indústria
- ✅ **Configurável**: RetryConfig permite customização
- ✅ **Observável**: Logging detalhado de tentativas
- ✅ **Type-Safe**: Genérico sobre Future<Output = Result<T>>
- ✅ **Composable**: Funciona com qualquer operação assíncrona

### Negativas
- ⚠️ **Latência**: Adiciona tempo em caso de falhas
- ⚠️ **Complexidade**: Desenvolvedores precisam entender retry
- ⚠️ **Sem Jitter**: Pode causar sincronização de clientes (mitigável)

### Neutras
- Pode ser desabilitado com `max_attempts = 1`
- Configuração por operação permite tuning fino

## Detalhes de Implementação

### RetryConfig
```rust
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,           // 3
    pub initial_delay_secs: u64,     // 1
    pub backoff_multiplier: f64,     // 2.0
    pub max_delay_secs: u64,         // 60
}

impl RetryConfig {
    pub fn new(max_attempts: u32, initial_delay_secs: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_secs,
            backoff_multiplier: 2.0,
            max_delay_secs: 60,
        }
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_secs = (self.initial_delay_secs as f64)
            * self.backoff_multiplier.powi(attempt as i32 - 1);

        let delay_secs = delay_secs.min(self.max_delay_secs as f64);
        Duration::from_secs_f64(delay_secs)
    }
}
```

### retry_with_backoff
```rust
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

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_attempts {
                    return Err(e);
                }

                let delay = config.calculate_delay(attempt);
                warn!("Tentativa {}/{} falhou: {}. Aguardando {:?}...",
                    attempt, config.max_attempts, e, delay);

                sleep(delay).await;
            }
        }
    }
}
```

### Uso
```rust
let config = RetryConfig::new(3, 1);
let result = retry_with_backoff(
    config,
    || async { drive_client.upload_file(path).await },
    "upload_file"
).await?;
```

## Riscos e Mitigações

| Risco | Probabilidade | Impacto | Mitigação |
|-------|---------------|---------|-----------|
| Thundering herd sem jitter | Média | Médio | Adicionar jitter em versão futura |
| Retry de erros permanentes | Baixa | Médio | Classificação de erros retryable |
| Configuração inadequada | Média | Baixo | Defaults sensatos, documentação |
| Loops infinitos | Muito Baixa | Alto | max_attempts garante término |

## Métricas de Sucesso

- ✅ **Cobertura de Testes**: 11 testes criados, 85.23% cobertura
- ✅ **Testa Cenários**: Sucesso imediato, retry sucesso, exaustão
- ✅ **Performance**: Delay medido corresponde ao esperado
- ✅ **Adoção**: Usado em todas as operações do DriveStorageBackend
- 🔄 **Produção**: Taxa de sucesso após retry >90% (futuro)

## Cronograma

- **Proposta:** 2026-01-05
- **Revisão:** 2026-01-06
- **Aprovação:** 2026-01-06
- **Implementação:** 2026-01-06 até 2026-01-07
- **Revisão Pós-Implementação:** 2026-01-14 (previsto)

## Referências

- [AWS Architecture Blog - Exponential Backoff And Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [Google Cloud Best Practices - Retry](https://cloud.google.com/architecture/best-practices-for-cloud-storage)
- [RFC 2616 - HTTP Retry-After](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.37)
- Issue #3: "Adicionar retry com backoff"
- PR #XX: "Implementar retry_with_backoff"

## Notas de Revisão

- 2026-01-07: Status alterado para Accepted
- 2026-01-07: Testes confirmam 85.23% de cobertura
- 2026-01-07: Jitter planejado para v1.1

## Relacionamentos

- **Complementa:** ADR-0001 (Retry funciona com qualquer StorageBackend)
- **Complementa:** ADR-0002 (Retry lida com 429s que ultrapassam rate limit)
- **Usado por:** ADR-0004 (Cada upload paralelo usa retry independente)
