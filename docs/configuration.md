# Referência de Configuração

## Estrutura do Arquivo config.toml

### [general]

Configurações gerais da aplicação.

```toml
[general]
log_level = "info"              # trace, debug, info, warn, error
log_file = "./logs/app.log"     # Opcional, omitir para stdout apenas
log_format = "text"             # text ou json
```

### [source]

Configuração da pasta de origem.

```toml
[source]
path = "/caminho/absoluto"      # Pasta a sincronizar
recursive = true                 # Incluir subpastas
ignore_hidden = true             # Ignorar arquivos iniciados com .
follow_symlinks = false          # Seguir links simbólicos
ignore_patterns = [              # Padrões glob para ignorar
    "*.tmp",
    "node_modules/",
    ".git/"
]
```

### [google_drive]

Configuração do Google Drive.

```toml
[google_drive]
credentials_file = "./credentials.json"
token_file = "./token.json"
target_folder_id = "1ABC123"    # ID da pasta (opção 1)
# OU
target_folder_name = "Backup"   # Nome da pasta (opção 2)

scopes = [                       # Escopos OAuth2
    "https://www.googleapis.com/auth/drive.file"
]
```

### [sync]

Configuração de sincronização.

```toml
[sync]
mode = "once"                    # once ou watch
interval_seconds = 300           # Intervalo em modo watch
conflict_resolution = "overwrite" # overwrite, skip ou rename
max_file_size_mb = 100          # Tamanho máximo por arquivo
chunk_size_mb = 5               # Tamanho do chunk para upload
max_concurrent_uploads = 4      # Uploads simultâneos
verify_upload = true            # Verificar MD5 após upload
preserve_folder_structure = true # Manter estrutura de pastas
delete_remote_on_local_delete = false # CUIDADO!
```

### [retry]

Configuração de tentativas.

```toml
[retry]
max_attempts = 3
initial_delay_seconds = 1
backoff_multiplier = 2.0        # Delay dobra a cada tentativa
max_delay_seconds = 60
```

### [notifications]

Configuração de notificações (opcional).

```toml
[notifications]
enabled = false
on_complete = true
on_error = true
notification_type = "desktop"   # desktop ou webhook
webhook_url = "https://..."     # Se type = webhook
```

### [state]

Configuração de estado.

```toml
[state]
state_file = "./state.json"
save_interval = 10              # Salvar a cada N arquivos
cleanup_after_days = 30         # Limpar registros antigos
```

## Valores Padrão

Se omitidos, os seguintes valores serão usados:

| Configuração | Padrão |
|-------------|--------|
| log_level | "info" |
| log_format | "text" |
| recursive | true |
| ignore_hidden | true |
| mode | "once" |
| conflict_resolution | "overwrite" |
| max_concurrent_uploads | 4 |
| max_attempts | 3 |

## Variáveis de Ambiente

Você pode sobrescrever configurações via variáveis de ambiente:

```bash
export RUSTDRIVESYNC_LOG_LEVEL=debug
export RUST_LOG=rustdrivesync=debug  # Alternativa
```

## Validação

Para validar sua configuração:

```bash
rustdrivesync config --validate
```

Isso verificará:
- Pasta de origem existe
- Arquivos de credenciais existem
- Valores estão em faixas válidas
- Modos e opções são válidos
