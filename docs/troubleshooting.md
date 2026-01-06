# Troubleshooting

## Problemas Comuns

### 1. "Arquivo de configuração não encontrado"

**Causa**: O arquivo `config.toml` não existe.

**Solução**:
```bash
rustdrivesync config --init
cp config.example.toml config.toml
# Edite config.toml com suas configurações
```

### 2. "Token expirado e não foi possível renovar"

**Causa**: O refresh token expirou ou foi revogado.

**Solução**:
```bash
rustdrivesync auth  # Re-autenticar
```

### 3. "Credenciais não encontradas"

**Causa**: Arquivo `credentials.json` não existe ou caminho incorreto.

**Solução**:
1. Baixe credenciais do Google Cloud Console
2. Renomeie para `credentials.json`
3. Coloque na raiz do projeto ou atualize o caminho em `config.toml`

### 4. "Pasta de destino não encontrada no Drive"

**Causa**: O `target_folder_id` está incorreto.

**Solução**:
- Verifique o ID na URL do Google Drive
- OU use `target_folder_name` em vez de `target_folder_id`

### 5. "Permissão negada" ao ler arquivos

**Causa**: Sem permissão para ler a pasta de origem.

**Solução**:
```bash
ls -la /caminho/para/pasta  # Verificar permissões
chmod u+r /caminho/para/pasta/*  # Dar permissão de leitura
```

### 6. "Rate limit excedido"

**Causa**: Muitas requisições à API do Google Drive.

**Solução**:
- Aguarde o tempo indicado na mensagem
- Reduza `max_concurrent_uploads` em `config.toml`
- Aumente `interval_seconds` em modo watch

### 7. "Upload falhou após N tentativas"

**Causas possíveis**:
- Problema de rede
- Arquivo muito grande
- Timeout

**Soluções**:
```toml
# Aumentar tentativas e delays
[retry]
max_attempts = 5
max_delay_seconds = 120

# Reduzir tamanho do chunk
[sync]
chunk_size_mb = 2
```

### 8. "Cota do Google Drive excedida"

**Causa**: Sem espaço no Google Drive.

**Solução**:
- Limpar espaço no Google Drive
- Upgrade para plano maior
- Usar `max_file_size_mb` para limitar uploads

### 9. Aplicação muito lenta

**Causas e Soluções**:

```toml
# 1. Muitos arquivos ignorados
[source]
ignore_patterns = ["*.tmp", "node_modules/"]  # Seja específico

# 2. Muitos uploads simultâneos
[sync]
max_concurrent_uploads = 2  # Reduzir

# 3. Chunks muito pequenos
[sync]
chunk_size_mb = 10  # Aumentar (para conexões rápidas)
```

### 10. Logs não aparecem

**Solução**:
```bash
# Modo verbose
rustdrivesync --verbose sync

# Ou via variável de ambiente
export RUST_LOG=rustdrivesync=debug
rustdrivesync sync
```

## Debug Avançado

### Habilitar logs detalhados

```toml
[general]
log_level = "trace"
log_file = "./debug.log"
```

### Ver requisições HTTP

```bash
export RUST_LOG=rustdrivesync=debug,reqwest=debug
rustdrivesync sync --verbose
```

### Testar sem fazer uploads

```bash
rustdrivesync sync --dry-run
```

## Logs de Erro

Os logs de erro incluem:
- Timestamp
- Nível de severidade
- Mensagem descritiva
- Path do arquivo (quando aplicável)

Exemplo:
```
2024-01-15 14:32:15 ERROR rustdrivesync: Upload falhou após 3 tentativas: arquivo.txt
```

## Reportar Bugs

Se o problema persistir:

1. Habilite logs detalhados (`log_level = "debug"`)
2. Reproduza o problema
3. Abra uma issue em: https://github.com/usuario/rustdrivesync/issues
4. Inclua:
   - Versão do rustdrivesync (`--version`)
   - Sistema operacional
   - Arquivo de log (sem credenciais!)
   - Passos para reproduzir

## FAQ

**P: Posso sincronizar múltiplas pastas?**
R: Atualmente não. Use múltiplas instâncias com configs diferentes.

**P: Há sincronização bidirecional?**
R: Não no MVP. Planejado para v2.0.

**P: Posso pausar a sincronização?**
R: Em modo watch, use Ctrl+C. Em modo once, aguarde a conclusão.

**P: Os arquivos são criptografados?**
R: Não no MVP. Os arquivos usam a criptografia padrão do Google Drive.
