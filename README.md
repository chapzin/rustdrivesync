# RustDriveSync

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Status](https://img.shields.io/badge/status-production--ready-brightgreen)
![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Tests](https://img.shields.io/badge/tests-113%20passing-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-36.46%25-yellow)

**Sincronização unidirecional de arquivos com Google Drive**

RustDriveSync é uma ferramenta CLI desenvolvida em Rust para sincronização eficiente e confiável de arquivos locais com o Google Drive. Projetada para ser leve, rápida e segura.

## 🎯 Características

### Core Features (V1.0)
- ✅ **Dependency Injection** - Abstrações via traits para múltiplos backends
- ✅ **Rate Limiting** - Proteção automática contra limites da API (800 req/100s)
- ✅ **Retry com Backoff** - Recuperação automática de falhas (exponential backoff)
- ✅ **Uploads Paralelos** - Até 10x mais rápido (configurável)
- ✅ **Thread-Safe** - Arquitetura async/await com Tokio
- ✅ **Sincronização unidirecional** (local → Google Drive)
- ✅ **Modo único ou monitoramento contínuo** (watch mode)
- ✅ **Upload com streaming** - memória constante para arquivos grandes
- ✅ **Autenticação OAuth2** segura
- ✅ **Configuração via arquivo TOML**
- ✅ **Logs detalhados** com níveis configuráveis
- ✅ **Detecção de mudanças** por hash MD5 incremental
- ✅ **Detecção automática de MIME types** - suporte a 800+ formatos

### Qualidade e Arquitetura
- ✅ **113 testes** (36.46% cobertura total, >70% em módulos críticos)
- ✅ **Documentação completa** - ADRs, C4 diagrams, security docs
- ✅ **SOLID principles** - Clean architecture
- ✅ **Zero breaking changes** - Código legado continua funcionando

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

### 1. Obter Credenciais do Google Cloud Console

Para usar o RustDriveSync, você precisa de credenciais OAuth 2.0 do Google. Siga este guia passo a passo:

#### Passo 1: Criar um Projeto no Google Cloud

1. **Acesse o Google Cloud Console**
   - URL: https://console.cloud.google.com
   - Faça login com sua conta Google

2. **Criar Novo Projeto**
   - Clique no seletor de projeto no topo (ao lado de "Google Cloud")
   - Clique em **"New Project"** (Novo Projeto)
   - Digite um nome: ex: "RustDriveSync" ou "My Drive Backup"
   - Clique em **"Create"** (Criar)
   - Aguarde alguns segundos até o projeto ser criado
   - Selecione o projeto criado no seletor de projetos

#### Passo 2: Habilitar a Google Drive API

1. **Acessar a API Library**
   - No menu lateral (☰), vá em: **"APIs & Services"** → **"Library"**
   - Ou acesse diretamente: https://console.cloud.google.com/apis/library

2. **Buscar e Habilitar**
   - Na barra de busca, digite: `Google Drive API`
   - Clique em **"Google Drive API"** nos resultados
   - Clique no botão azul **"Enable"** (Habilitar)
   - Aguarde a ativação (geralmente instantânea)

#### Passo 3: Configurar Tela de Consentimento OAuth

⚠️ **Obrigatório antes de criar credenciais**

1. **Acessar OAuth Consent Screen**
   - No menu lateral: **"APIs & Services"** → **"OAuth consent screen"**
   - Ou: https://console.cloud.google.com/apis/credentials/consent

2. **Configurar Tipo de Usuário**
   - Selecione **"External"** (para uso pessoal)
   - Clique em **"Create"** (Criar)

3. **Preencher Informações Obrigatórias**
   - **App name**: "RustDriveSync" (ou seu nome preferido)
   - **User support email**: Seu email
   - **Developer contact email**: Seu email
   - Clique em **"Save and Continue"** (Salvar e Continuar)

4. **Escopos (Scopes)**
   - Clique em **"Add or Remove Scopes"**
   - Busque e selecione: `https://www.googleapis.com/auth/drive.file`
   - Clique em **"Update"** e depois **"Save and Continue"**

5. **Test Users (Usuários de Teste)**
   - Clique em **"Add Users"**
   - Adicione seu email (o que você usará para sincronizar)
   - Clique em **"Add"** e depois **"Save and Continue"**

6. **Revisar e Confirmar**
   - Clique em **"Back to Dashboard"**

#### Passo 4: Criar Credenciais OAuth 2.0

1. **Acessar Credentials**
   - No menu lateral: **"APIs & Services"** → **"Credentials"**
   - Ou: https://console.cloud.google.com/apis/credentials

2. **Criar Nova Credencial**
   - Clique no botão **"+ Create Credentials"** no topo
   - Selecione **"OAuth client ID"**

3. **Configurar Tipo de Aplicação**
   - **Application type**: Selecione **"Desktop app"**
   - **Name**: "RustDriveSync CLI" (ou qualquer nome)
   - Clique em **"Create"**

4. **Baixar Credenciais**
   - Uma janela popup aparecerá com "OAuth client created"
   - Clique em **"Download JSON"** (ícone de download ⬇️)
   - O arquivo será baixado como `client_secret_XXXXX.json`
   - **Renomeie** este arquivo para `credentials.json`
   - **Mova** para uma pasta segura (ex: `~/.config/rustdrivesync/`)

#### Passo 5: Proteger Suas Credenciais

```bash
# Linux/macOS
mkdir -p ~/.config/rustdrivesync
mv ~/Downloads/client_secret_*.json ~/.config/rustdrivesync/credentials.json
chmod 600 ~/.config/rustdrivesync/credentials.json

# Windows (PowerShell)
New-Item -Path "$env:APPDATA\rustdrivesync" -ItemType Directory -Force
Move-Item "$env:USERPROFILE\Downloads\client_secret_*.json" "$env:APPDATA\rustdrivesync\credentials.json"
```

✅ **Pronto!** Você agora tem o arquivo `credentials.json` necessário.

---

### ⚠️ Troubleshooting

**Erro: "Access blocked: This app's request is invalid"**
- Certifique-se de configurar a OAuth Consent Screen (Passo 3)
- Adicione seu email como Test User

**Erro: "The OAuth client was not found"**
- Verifique se criou credenciais do tipo "Desktop app"
- Não use "Web application" ou "Service account"

**Arquivo credentials.json não baixa**
- Vá em Credentials → Clique no nome da credencial criada
- Clique no ícone de download (⬇️) no lado direito

---

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

## 📚 Documentação

### Para Usuários
- **[Guia de Uso Completo](GUIA_DE_USO.md)** - Tutorial passo a passo em Português
  - Instalação e configuração inicial
  - Comandos básicos e avançados
  - Exemplos práticos de uso
  - Solução de problemas comuns
  - Dicas de segurança

### Para Desenvolvedores
- **[Sumário de Arquitetura](docs/ARCHITECTURE_SUMMARY.md)** - Visão executiva da arquitetura
- **[Documentação Completa de Arquitetura](docs/architecture/README.md)** - Índice central
  - **[ADRs - Architecture Decision Records](docs/architecture/adr/)**
    - [0001: Dependency Injection com Traits](docs/architecture/adr/0001-dependency-injection-with-traits.md)
    - [0002: Rate Limiting para Google Drive API](docs/architecture/adr/0002-rate-limiting-google-drive-api.md)
    - [0003: Retry com Exponential Backoff](docs/architecture/adr/0003-retry-with-exponential-backoff.md)
    - [0004: Uploads Paralelos](docs/architecture/adr/0004-parallel-uploads.md)
  - **[Diagramas C4](docs/architecture/diagrams/)**
    - [Context Diagram](docs/architecture/diagrams/c4-context.md) - Visão de sistema
    - [Container Diagram](docs/architecture/diagrams/c4-container.md) - Componentes internos
    - [Data Flow](docs/architecture/diagrams/data-flow.md) - Fluxo de dados
  - **[Atributos de Qualidade](docs/architecture/quality-attributes.md)** - Performance, Segurança, Confiabilidade
  - **[Arquitetura de Segurança](docs/architecture/security-architecture.md)** - Modelo de ameaças e compliance

### API Documentation
```bash
# Gerar documentação Rust
cargo doc --open
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

### ✅ V1.0 - Production Ready (CONCLUÍDO)

**Fundação**:
- [x] Estrutura base e CLI ✅
- [x] Sistema de configuração ✅
- [x] Integração com Google Drive ✅
- [x] Sync engine completo ✅
- [x] File watcher (modo watch) ✅
- [x] Streaming de uploads ✅

**Qualidade e Performance**:
- [x] Dependency Injection com traits ✅
- [x] Rate limiting automático (800 req/100s) ✅
- [x] Retry com exponential backoff ✅
- [x] Uploads paralelos (5-10x speedup) ✅
- [x] 113 testes (>70% cobertura em módulos críticos) ✅
- [x] Documentação completa (ADRs, C4, Security) ✅

### 🚀 Próximos Passos

**V1.1 (Q1 2026)** - Melhorias de Segurança e Observabilidade:
- [ ] OS Keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- [ ] Jitter em retry para prevenir thundering herd
- [ ] Structured logging (JSON format)
- [ ] Prometheus metrics endpoint
- [ ] E2E tests com Docker

**V1.2 (Q2 2026)** - Múltiplos Backends:
- [ ] Backend S3 (AWS S3, MinIO, Wasabi)
- [ ] Backend Dropbox
- [ ] SQLite state (substituir JSON)
- [ ] Progress bar melhorado (multi-file)
- [ ] Cobertura de testes >50%

**V2.0 (Q3-Q4 2026)** - Sincronização Avançada:
- [ ] Sincronização bidirecional (Drive → Local)
- [ ] Resolução automática de conflitos
- [ ] End-to-end encryption opcional
- [ ] Web dashboard (status, métricas, logs)
- [ ] Multi-user support

**V3.0 (2027)** - Enterprise Features:
- [ ] Webhooks para notificações
- [ ] API REST para integração
- [ ] Suporte a múltiplos clouds simultâneos
- [ ] Versionamento de arquivos
- [ ] Rollback de mudanças

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
