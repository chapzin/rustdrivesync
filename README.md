# RustDriveSync

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Status](https://img.shields.io/badge/status-beta-yellow)
![Version](https://img.shields.io/badge/version-0.1.0-blue)

**Sincronização unidirecional de arquivos com Google Drive**

RustDriveSync é uma ferramenta CLI desenvolvida em Rust para sincronização eficiente e confiável de arquivos locais com o Google Drive. Projetada para ser leve, rápida e segura.

## 🎯 Características

- ✅ **Sincronização unidirecional** (local → Google Drive)
- ✅ **Modo único ou monitoramento contínuo** (watch mode)
- ✅ **Upload com streaming** - memória constante (~5MB) para arquivos de qualquer tamanho
- ✅ **Autenticação OAuth2** segura
- ✅ **Configuração via arquivo TOML**
- ✅ **Logs detalhados** com níveis configuráveis
- ✅ **Detecção de mudanças** por hash MD5 incremental
- ✅ **Uso eficiente de memória** - ~5MB RAM constante, independente do tamanho do arquivo
- ✅ **Detecção automática de MIME types** - suporte a 800+ formatos

## 📋 Pré-requisitos

- Rust 1.70 ou superior
- Credenciais OAuth2 do Google Cloud Console

## 🚀 Instalação

### Via Cargo

```bash
cargo install rustdrivesync
```

### Build manual

```bash
git clone https://github.com/usuario/rustdrivesync
cd rustdrivesync
cargo build --release
```

O binário estará em `target/release/rustdrivesync`

## 🔧 Configuração Inicial

### 1. Obter Credenciais do Google

1. Acesse o [Google Cloud Console](https://console.cloud.google.com)
2. Crie um novo projeto (ou use um existente)
3. Habilite a **Google Drive API**
4. Crie credenciais OAuth 2.0 (Desktop App)
5. Baixe o arquivo `credentials.json`

### 2. Criar Configuração

```bash
rustdrivesync config --init
```

Isso criará um arquivo `config.example.toml`. Copie-o para `config.toml` e edite:

```bash
cp config.example.toml config.toml
vim config.toml
```

Configure pelo menos:
- `source.path`: pasta local a sincronizar
- `google_drive.credentials_file`: caminho para credentials.json
- `google_drive.target_folder_id` ou `target_folder_name`

### 3. Autenticar

```bash
rustdrivesync auth
```

Isso abrirá seu navegador para autorizar o aplicativo.

## 📖 Uso

### Sincronização Única

```bash
rustdrivesync sync --once
```

### Monitoramento Contínuo

```bash
rustdrivesync sync --watch
```

### Simular (Dry Run)

```bash
rustdrivesync sync --dry-run
```

### Verificar Status

```bash
rustdrivesync status
```

### Listar Arquivos

```bash
# Arquivos pendentes
rustdrivesync list

# Arquivos no Google Drive
rustdrivesync list --remote

# Diferenças local vs remoto
rustdrivesync list --diff
```

## ⚙️ Configuração

Exemplo de `config.toml`:

```toml
[general]
log_level = "info"

[source]
path = "/home/usuario/documentos"
recursive = true
ignore_hidden = true
ignore_patterns = ["*.tmp", "node_modules/", ".git/"]

[google_drive]
credentials_file = "./credentials.json"
token_file = "./token.json"
target_folder_id = "1ABC123xyz"

[sync]
mode = "watch"
interval_seconds = 300
conflict_resolution = "overwrite"
max_file_size_mb = 100
chunk_size_mb = 5
```

Veja `config.example.toml` para todas as opções disponíveis.

## 🏗️ Arquitetura

```
┌─────────────────────────────────────────────────┐
│                  RustDriveSync                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  CLI (clap) → Config → Core Engine              │
│                         │                       │
│                         ├─ File Scanner         │
│                         ├─ Sync Engine          │
│                         └─ State Manager        │
│                         │                       │
│                         ▼                       │
│               Google Drive Client               │
│               (OAuth2 + Upload)                 │
│                                                 │
└─────────────────────────────────────────────────┘
```

## 🧪 Desenvolvimento

### Build

```bash
cargo build
```

### Testes

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatação

```bash
cargo fmt
```

## 📝 Roadmap

### Status Atual: V0.1.0 (Em desenvolvimento ativo)

**Concluído**:
- [x] **Fase 1**: Estrutura base e CLI ✅
- [x] **Fase 2**: Sistema de configuração ✅
- [x] **Fase 3**: Integração com Google Drive ✅
- [x] **Fase 4**: Sync engine completo ✅
- [x] **Fase 5**: File watcher (modo watch) ✅
- [x] **Streaming de uploads** - Memória constante para arquivos grandes ✅

**Próximos Passos** (V1.0 - Production Ready):
- [ ] Rate limiting e circuit breaker
- [ ] Retry com exponential backoff
- [ ] Uploads concorrentes (5-10x mais rápido)
- [ ] Commands `/status` e `/list` completos
- [ ] Testes de integração E2E
- [ ] Documentação completa

**Visão de Longo Prazo**:
- **V2.0**: Sincronização bidirecional, versionamento, exclusão de arquivos
- **V3.0**: Dashboard web, webhooks, suporte a múltiplos clouds

📋 Veja [ROADMAP.md](ROADMAP.md) para detalhes completos sobre features planejadas, sprints, e debt técnica.
📄 Veja [CHANGELOG.md](CHANGELOG.md) para histórico detalhado de mudanças.

## 🤝 Contribuindo

Contribuições são bem-vindas! Por favor:

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 📄 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 🔗 Links Úteis

- [Documentação da Google Drive API](https://developers.google.com/drive/api)
- [OAuth 2.0 para Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app)
- [Documentação Rust](https://doc.rust-lang.org/)

## 🙏 Agradecimentos

- Comunidade Rust
- Mantenedores das crates utilizadas (clap, tokio, yup-oauth2, etc.)
- Contribuidores do projeto

---

**Desenvolvido com ❤️ em Rust**
