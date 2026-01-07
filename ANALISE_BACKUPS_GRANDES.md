# 📊 Análise: Backups SQL Grandes (6-8 GB)

## 🎯 Cenário do Usuário

- **Arquivos grandes**: 6-8 GB cada
- **Frequência**: ~4 arquivos por dia = 24-32 GB/dia
- **Adicionais**: Diversos arquivos menores
- **Total estimado**: ~30-35 GB/dia

---

## ⚠️ PROBLEMAS IDENTIFICADOS

### 1. **Limitação na Configuração Atual** ❌

**CRÍTICO**: A configuração padrão bloqueia arquivos grandes!

```toml
# config.example.toml - LINHA 105
max_file_size_mb = 100  # ❌ Bloqueia arquivos > 100 MB!
```

**Seus arquivos**: 6.000-8.000 MB
**Limite atual**: 100 MB
**Resultado**: ❌ Uploads serão **rejeitados**!

### 2. **Chunk Size Subótimo**

```toml
chunk_size_mb = 5  # Para arquivos grandes, pode ser otimizado
```

---

## ✅ LIMITES DO GOOGLE DRIVE API

### Limites Diários (Google Workspace)
| Limite | Valor | Status |
|--------|-------|--------|
| **Upload diário** | 750 GB/dia | ✅ OK (você usa ~30 GB) |
| **Requisições/100s** | 800 req/100s | ✅ OK (RustDriveSync tem rate limiting) |
| **Tamanho máximo por arquivo** | 5 TB | ✅ OK (seus arquivos: 6-8 GB) |
| **Sessão resumível** | 1 semana | ✅ OK |

**Veredito**: Seus volumes estão **DENTRO dos limites** do Google Drive! 🎉

---

## 🔧 CONFIGURAÇÃO RECOMENDADA

### Para Backups SQL de 6-8 GB:

```toml
[sync]
# Aumentar limite de tamanho (aceitar até 10GB)
max_file_size_mb = 10240  # 10 GB

# Chunk maior para uploads mais eficientes
chunk_size_mb = 32  # 32 MB (recomendado para arquivos grandes)

# Reduzir uploads simultâneos (arquivos grandes consomem mais banda)
max_concurrent_uploads = 2  # Evita saturar conexão

# Sempre usar resumable
verify_upload = true

# Preservar estrutura de pastas (opcional)
preserve_folder_structure = true

[retry]
# Aumentar tentativas (uploads grandes têm mais chance de falhar)
max_attempts = 5

# Aumentar delays (dar mais tempo para recuperação)
initial_delay_seconds = 2
backoff_multiplier = 2.0
max_delay_seconds = 120
```

---

## 📈 PERFORMANCE ESPERADA

### Upload de 7 GB com Configuração Otimizada

**Conexão**: 100 Mbps = 12.5 MB/s teórico

| Métrica | Valor Estimado |
|---------|----------------|
| **Velocidade real** | ~8-10 MB/s (overhead HTTP/TLS) |
| **Tempo por arquivo** | 11-14 minutos |
| **4 arquivos/dia** | ~50 minutos total |
| **Memória usada** | ~256 KB fixo (streaming) |

### Gargalos Potenciais:
1. **Velocidade de upload** (mais provável)
2. **Latência da rede**
3. **Cálculo de MD5** (~30s para 7GB)

---

## 🚨 POSSÍVEIS PROBLEMAS E SOLUÇÕES

### Problema 1: Upload Interrompido

**Causa**: Conexão instável, timeout
**Solução**: ✅ RustDriveSync usa **Resumable Upload**
- Retoma de onde parou
- Não recomeça do zero
- Sessão válida por 1 semana

### Problema 2: MD5 Calculation Lento

**Causa**: Cálculo de hash de 7GB é demorado
**Status Atual**: ✅ Implementado com streaming (256KB chunks)
```rust
// src/google_drive/upload.rs:223
let mut buffer = vec![0u8; 256 * 1024]; // 256KB chunks
```
**Tempo estimado**: ~30 segundos para 7GB em SSD

### Problema 3: Memória Insuficiente

**Status**: ✅ **Resolvido por design**!
```rust
// Upload usa streaming, não carrega arquivo inteiro
const CHUNK_SIZE: usize = 256 * 1024; // 256KB
let stream = ReaderStream::with_capacity(file, CHUNK_SIZE);
```
**Memória usada**: ~256 KB constante (independente do tamanho do arquivo)

### Problema 4: Rate Limiting

**Status**: ✅ **Protegido**
```rust
// RustDriveSync tem rate limiter embutido
// Limite: 800 requisições/100s (Google Drive)
```

---

## ⚡ OTIMIZAÇÕES ADICIONAIS

### 1. Compressão (Opcional)

**Antes do Upload**:
```bash
# Comprimir backup antes de enviar
gzip -9 backup.sql  # Reduz 70-90% do tamanho
# backup.sql (7GB) → backup.sql.gz (~1GB)
```

**Vantagens**:
- ✅ Upload 7x mais rápido
- ✅ Economia de espaço no Drive
- ✅ Menos banda consumida

**Desvantagens**:
- ❌ Tempo extra de compressão (~2-3 min)
- ❌ CPU adicional

### 2. Upload Noturno/Off-peak

```toml
[sync]
mode = "watch"
interval_seconds = 3600  # Verificar a cada hora

# Usar cron/scheduler para rodar apenas à noite
# Exemplo: 2h da manhã quando banda está ociosa
```

### 3. Múltiplas Pastas no Drive

```toml
[sync]
preserve_folder_structure = true

# Organizar por data
# RustDriveSync/backups/2026-01-07/database1.sql
# RustDriveSync/backups/2026-01-07/database2.sql
```

---

## 📋 CHECKLIST DE IMPLEMENTAÇÃO

### Passo 1: Atualizar Configuração
```bash
vim config.toml
# Alterar max_file_size_mb = 10240
# Alterar chunk_size_mb = 32
# Alterar max_concurrent_uploads = 2
```

### Passo 2: Testar com Arquivo Real
```bash
# Dry run primeiro
rustdrivesync sync --dry-run

# Upload real
rustdrivesync sync --once
```

### Passo 3: Monitorar Logs
```toml
[general]
log_level = "info"  # Ver progresso
log_file = "./logs/rustdrivesync.log"
```

### Passo 4: Automatizar
```bash
# Crontab (Linux)
0 2 * * * /usr/local/bin/rustdrivesync sync --once >> /var/log/backup-sync.log 2>&1
```

---

## 🔍 MONITORAMENTO

### Métricas Importantes:

1. **Tempo de Upload**: 11-14 min/arquivo esperado
2. **Taxa de Sucesso**: Deve ser > 95%
3. **Uso de Banda**: ~8-10 MB/s durante upload
4. **Espaço no Drive**: Monitorar quota

### Comando de Status:
```bash
rustdrivesync status
```

---

## 📚 REFERÊNCIAS

- [Google Drive API Usage Limits](https://developers.google.com/workspace/drive/api/guides/limits)
- [Resumable Upload Protocol](https://developers.google.com/drive/api/guides/manage-uploads)
- [Storage Limits](https://support.google.com/a/answer/172541?hl=en)

---

## ✅ CONCLUSÃO

### Seu cenário é **VIÁVEL** ✅

| Aspecto | Status |
|---------|--------|
| Volume diário (30 GB) | ✅ Dentro do limite (750 GB) |
| Tamanho por arquivo (6-8 GB) | ✅ Dentro do limite (5 TB) |
| Memória necessária | ✅ Apenas 256 KB |
| Suporte a interrupções | ✅ Resumable upload |
| Rate limiting | ✅ Protegido |

### Ações Necessárias:

1. ✅ **CRÍTICO**: Aumentar `max_file_size_mb` para 10240
2. ✅ **Recomendado**: Aumentar `chunk_size_mb` para 32
3. ✅ **Recomendado**: Reduzir `max_concurrent_uploads` para 2
4. ✅ **Opcional**: Considerar compressão com gzip

### Tempo Total Estimado:
- **4 arquivos de 7GB**: ~50 minutos/dia
- **Arquivos menores**: +10-15 minutos
- **Total**: ~1 hora/dia de upload

**Recomendação**: ✅ Use o RustDriveSync com a configuração ajustada. O sistema está preparado para esse cenário!
