# Guia de Uso - RustDriveSync

## 🚀 Como Usar o RustDriveSync

### Pré-requisitos

Antes de começar, você precisa:

1. ✅ **Rust instalado** (1.70 ou superior)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. ✅ **Credenciais do Google Drive API**
   - Acesse o [Google Cloud Console](https://console.cloud.google.com/)
   - Crie um projeto
   - Habilite a Google Drive API
   - Crie credenciais OAuth 2.0 (Desktop App)
   - Baixe o arquivo `credentials.json`

3. ✅ **Conexão com internet**

---

## 📦 Instalação

### Opção 1: Compilar do Código-Fonte

```bash
# Clone o repositório
git clone https://github.com/seu-usuario/rust-driver.git
cd rust-driver

# Compile em modo release
cargo build --release

# O executável estará em:
./target/release/rustdrivesync
```

### Opção 2: Instalar via Cargo (Futuro)

```bash
cargo install rustdrivesync
```

---

## ⚙️ Configuração Inicial

### 1. Crie o Arquivo de Configuração

```bash
mkdir -p ~/.config/rustdrivesync
```

Crie o arquivo `~/.config/rustdrivesync/config.toml`:

```toml
[general]
log_level = "info"                    # debug, info, warn, error
log_file = "~/.config/rustdrivesync/sync.log"
log_format = "text"                   # text ou json

[source]
path = "/home/seu-usuario/Documents"  # Diretório a sincronizar
recursive = true                       # Incluir subdiretórios
ignore_hidden = false                  # Ignorar arquivos ocultos
follow_symlinks = false                # Seguir links simbólicos
ignore_patterns = [                    # Padrões a ignorar
    ".git",
    "node_modules",
    "*.tmp",
    "*.swp"
]

[google_drive]
credentials_file = "~/.config/rustdrivesync/credentials.json"
token_file = "~/.config/rustdrivesync/token.json"
target_folder_name = "RustDriveSync Backup"  # Nome da pasta no Drive
scopes = [
    "https://www.googleapis.com/auth/drive.file"
]

[sync]
mode = "once"                          # "once" ou "watch"
interval_seconds = 300                 # Intervalo para re-scan (watch mode)
conflict_resolution = "overwrite"      # "overwrite", "skip", "rename"
max_file_size_mb = 100                 # Tamanho máximo (0 = sem limite)
chunk_size_mb = 5                      # Tamanho de chunks para upload
max_concurrent_uploads = 2             # Uploads simultâneos (1-10)
verify_upload = true                   # Verificar checksum após upload
preserve_folder_structure = true       # Manter estrutura de pastas
delete_remote_on_local_delete = false  # Deletar no Drive ao deletar local

[retry]
max_attempts = 3                       # Tentativas de retry
initial_delay_seconds = 1              # Delay inicial
backoff_multiplier = 2.0               # Multiplicador de backoff
max_delay_seconds = 60                 # Delay máximo

[notifications]
enabled = false                        # Notificações desktop
on_success = true                      # Notificar sucesso
on_failure = true                      # Notificar falhas
on_progress = false                    # Notificar progresso

[state]
state_file = "~/.config/rustdrivesync/state.json"
save_interval = 10                     # Salvar estado a cada N arquivos
cleanup_after_days = 30                # Limpar entradas antigas
```

### 2. Coloque as Credenciais

Copie o arquivo `credentials.json` baixado do Google Cloud Console:

```bash
cp ~/Downloads/credentials.json ~/.config/rustdrivesync/
chmod 600 ~/.config/rustdrivesync/credentials.json
```

---

## 🎯 Uso Básico

### Primeira Sincronização

```bash
# Sincronização única (modo manual)
./target/release/rustdrivesync sync

# Ou se instalado via cargo:
rustdrivesync sync
```

**O que acontece**:
1. Abre o navegador para autenticação OAuth
2. Você aprova o acesso
3. Sistema escaneia diretório local
4. Faz upload de todos os arquivos para Google Drive
5. Salva estado em `state.json`
6. Exibe estatísticas

**Saída esperada**:
```
[INFO] Inicializando engine de sincronização com Google Drive
[INFO] Pasta do Drive: RustDriveSync Backup (ID: 1AbCdEfGhIjKlMnOpQrStUvWxYz)
[INFO] Escaneando diretório: /home/user/Documents
[INFO] Encontrados 42 arquivos

Uploading files...
├─ document.pdf ━━━━━━━━━━━━━━━━━━━━━━━━ 100% (2.5 MB/s)
├─ photo.jpg ━━━━━━━━━━━━━━━━━━━━━━━━━━  100% (1.8 MB/s)
└─ video.mp4 ━━━━━━━━━━━━━━━━━━━━━━━━━━  100% (5.2 MB/s)

✅ Sincronização concluída!

Estatísticas:
- Arquivos escaneados: 42
- Arquivos novos: 42
- Arquivos atualizados: 0
- Falhas: 0
- Bytes enviados: 125.4 MB
- Tempo total: 18.3 segundos
- Throughput médio: 6.8 MB/s
```

### Sincronização Contínua (Watch Mode)

```bash
rustdrivesync watch
```

**O que acontece**:
1. Inicia monitoramento do diretório
2. Detecta mudanças em tempo real
3. Sincroniza automaticamente
4. Roda até você pressionar Ctrl+C

**Saída esperada**:
```
[INFO] Modo watch ativado
[INFO] Monitorando: /home/user/Documents
[INFO] Pressione Ctrl+C para parar

[INFO] Detectada mudança: /home/user/Documents/new-file.txt
[INFO] Iniciando sincronização...
[INFO] Upload concluído: new-file.txt (2.1 MB)

[INFO] Detectada mudança: /home/user/Documents/edited.txt
[INFO] Iniciando sincronização...
[INFO] Upload concluído: edited.txt (1.5 MB)
```

### Verificar Status

```bash
rustdrivesync status
```

**Saída esperada**:
```
Status da Sincronização
═══════════════════════════════════════

Configuração:
  Diretório local:  /home/user/Documents
  Pasta no Drive:   RustDriveSync Backup
  Modo:             once

Estado:
  Arquivos rastreados: 42
  Última sincronização: 2026-01-07 10:30:15
  Última sincronização completa: 2026-01-07 09:15:42

Pendências:
  Arquivos a sincronizar: 0
  Mudanças detectadas: 0

✅ Tudo sincronizado!
```

---

## 🔧 Comandos Avançados

### Sincronização com Config Customizada

```bash
rustdrivesync sync --config /path/to/custom-config.toml
```

### Modo Dry-Run (Simular sem fazer upload)

```bash
rustdrivesync sync --dry-run
```

**Útil para**:
- Testar configuração
- Ver o que seria sincronizado
- Debug

### Aumentar Verbosidade de Logs

```bash
# Debug mode
rustdrivesync sync --log-level debug

# Trace mode (muito verboso)
rustdrivesync sync --log-level trace
```

### Sincronizar Apenas Arquivos Específicos

```bash
# Edite config.toml para ajustar ignore_patterns
# ou use --include-pattern (futuro)
```

### Forçar Sincronização Completa

```bash
# Delete o state file
rm ~/.config/rustdrivesync/state.json

# Execute sync novamente
rustdrivesync sync
```

---

## 📊 Monitoramento

### Ver Logs em Tempo Real

```bash
# Terminal 1: Rodar sync
rustdrivesync watch

# Terminal 2: Seguir logs
tail -f ~/.config/rustdrivesync/sync.log
```

### Logs Estruturados (JSON)

Edite `config.toml`:
```toml
[general]
log_format = "json"
```

Agora você pode usar ferramentas como `jq`:
```bash
tail -f ~/.config/rustdrivesync/sync.log | jq '.message'
```

### Verificar Cobertura de Testes

```bash
cd rust-driver
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html
```

---

## ❓ Problemas Comuns

### 1. Erro de Autenticação

**Problema**:
```
Error: Falha na autenticação: invalid_grant
```

**Solução**:
```bash
# Delete tokens antigos
rm ~/.config/rustdrivesync/token.json

# Autentique novamente
rustdrivesync sync
```

### 2. Rate Limit Excedido

**Problema**:
```
Warning: Rate limit excedido. Aguardando 60 segundos...
```

**Solução**:
- ✅ Isso é normal! O sistema está funcionando corretamente
- ✅ Aguarde o backoff
- 🔧 Reduza `max_concurrent_uploads` no config se muito frequente

### 3. Arquivo Muito Grande

**Problema**:
```
Error: Arquivo muito grande: 150MB (máximo: 100MB)
```

**Solução**:
```toml
[sync]
max_file_size_mb = 0  # 0 = sem limite
```

### 4. State File Corrompido

**Problema**:
```
Error: Erro ao carregar state.json
```

**Solução**:
```bash
# Backup do state atual
cp ~/.config/rustdrivesync/state.json ~/.config/rustdrivesync/state.json.backup

# Usar backup automático
cp ~/.config/rustdrivesync/state.json.bak ~/.config/rustdrivesync/state.json

# Ou forçar full sync
rm ~/.config/rustdrivesync/state.json
rustdrivesync sync
```

### 5. Permissões Negadas

**Problema**:
```
Error: Permissão negada: /root/secret.txt
```

**Solução**:
- ✅ Não rode como root
- ✅ Verifique permissões do diretório
```bash
chmod +r /path/to/file
```

---

## 🎓 Exemplos de Uso

### Exemplo 1: Backup de Documentos Pessoais

```toml
[source]
path = "/home/user/Documents"
ignore_patterns = [".git", "*.tmp", "node_modules"]

[sync]
mode = "watch"
max_concurrent_uploads = 3
verify_upload = true
```

```bash
rustdrivesync watch &  # Roda em background
```

### Exemplo 2: Backup de Fotos (uma vez por dia)

```toml
[source]
path = "/home/user/Pictures"

[sync]
mode = "once"
max_file_size_mb = 0  # Sem limite (fotos grandes)
max_concurrent_uploads = 5
```

```bash
# Adicione ao crontab
crontab -e
# 0 2 * * * /usr/local/bin/rustdrivesync sync
```

### Exemplo 3: Backup de Código (sem node_modules)

```toml
[source]
path = "/home/user/Projects"
ignore_patterns = [
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    "*.log"
]

[sync]
mode = "watch"
max_concurrent_uploads = 2
```

---

## 🔐 Segurança

### Proteção de Credenciais

```bash
# Permissões corretas
chmod 600 ~/.config/rustdrivesync/credentials.json
chmod 600 ~/.config/rustdrivesync/token.json

# Nunca commite credenciais no Git
echo "credentials.json" >> .gitignore
echo "token.json" >> .gitignore
```

### Usar 2FA no Google

1. Acesse https://myaccount.google.com/security
2. Habilite "Verificação em duas etapas"
3. Use authenticator app (Google Authenticator, Authy)

### Revisar Permissões

```bash
# Ver apps com acesso ao Drive
# https://myaccount.google.com/permissions

# Revogar acesso se necessário
```

---

## 📖 Documentação Adicional

### Arquitetura
- [Sumário de Arquitetura](docs/ARCHITECTURE_SUMMARY.md)
- [Decisões Arquiteturais](docs/architecture/adr/)
- [Diagramas C4](docs/architecture/diagrams/)

### Segurança
- [Arquitetura de Segurança](docs/architecture/security-architecture.md)
- [Modelo de Ameaças](docs/architecture/security-architecture.md#modelo-de-ameaças)

### Desenvolvimento
- [Guia de Contribuição](CONTRIBUTING.md)
- [Melhorias V1.0](IMPLEMENTACOES_V1.md)

---

## 🆘 Suporte

### Reportar Bugs
- GitHub Issues: https://github.com/seu-usuario/rust-driver/issues
- Inclua: logs, config (sem credenciais), versão do Rust

### Dúvidas
- GitHub Discussions: https://github.com/seu-usuario/rust-driver/discussions
- Stack Overflow: tag `rustdrivesync`

### Contribuir
- Veja [CONTRIBUTING.md](CONTRIBUTING.md)
- PRs são bem-vindos!

---

## ✅ Checklist de Pós-Instalação

- [ ] Rust instalado e funcionando
- [ ] Credenciais do Google Drive configuradas
- [ ] Arquivo config.toml criado e customizado
- [ ] Primeira sincronização bem-sucedida
- [ ] Logs estão sendo gerados
- [ ] State file sendo atualizado
- [ ] Watch mode funcionando (se aplicável)
- [ ] Permissões de arquivo corretas (600)
- [ ] Backup do state.json feito

---

**Pronto! Você está sincronizando com o Google Drive! 🎉**

Para mais informações, consulte a [documentação completa](docs/ARCHITECTURE_SUMMARY.md).
