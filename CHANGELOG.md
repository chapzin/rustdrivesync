# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [Unreleased]

### Added
- Streaming de uploads com memória constante (~5MB) para arquivos grandes
- Cálculo incremental de MD5 para arquivos de qualquer tamanho
- Detecção de MIME type usando biblioteca `mime_guess` (800+ formatos)
- Método `get_token()` em `DriveClient` para requisições HTTP diretas
- Função `calculate_md5_streaming()` para hash assíncrono
- Função `upload_streaming()` usando Google Drive Resumable Upload Protocol
- Tipo de erro `StateError` para gerenciamento de estado
- Constante `FOLDER_MIME_TYPE` em `DriveClient`

### Changed
- Arquivos grandes (>=5MB) agora usam upload resumível com streaming
- `upload_resumable()` agora delega para `upload_streaming()`
- `prepare_file_metadata()` calcula MD5 apenas para arquivos < 5MB
- MIME type detection mais precisa e confiável
- `DriveClient` agora mantém referência ao `DriveAuthenticator`

### Fixed
- Removido `.unwrap()` perigoso em `src/google_drive/client.rs:63` (parse de MIME type)
- Removido `.unwrap()` perigoso em `src/sync/engine.rs:355` (max_file_size)
- Removido `.unwrap()` perigoso em `src/sync/engine.rs:379` (get_file)
- Correção de parse de MIME type em criação de pastas do Google Drive
- Pattern matching seguro com `if let Some()` em vez de `.unwrap()`

### Performance
- **Redução de 99.5% no uso de RAM** para arquivos grandes (1GB: de 1GB RAM → 5MB RAM)
- Upload de arquivos grandes agora escala para qualquer tamanho (testado até 10GB+)
- Chunks de 256KB otimizados para throughput e latência

### Security
- Eliminado risco de panic em produção por `.unwrap()`
- MIME type validation mais robusta com biblioteca mantida

---

## [0.1.0] - 2026-01-06

### Added
- Sincronização unidirecional local → Google Drive
- Autenticação OAuth2 com Google Drive API v3
- File watcher com debouncing de 2s usando `notify-debouncer-mini`
- Gerenciamento de estado persistente em JSON
- CLI com comandos: `sync`, `auth`, `config`
- Suporte a modo watch contínuo (`--watch`)
- Detecção automática de mudanças em arquivos
- Configuração via arquivo TOML
- Scanner de arquivos com ignore patterns
- Change tracker para detectar novos/modificados/deletados
- Upload de arquivos com verificação de MD5
- Criação automática de pastas no Google Drive
- Logs estruturados com `tracing`

### Features
- **Sync Engine**: Motor de sincronização robusto
- **File Watcher**: Monitoramento em tempo real de mudanças
- **State Manager**: Rastreamento de arquivos sincronizados
- **Drive Client**: Integração completa com Google Drive API
- **Config System**: Sistema de configuração flexível com validação

### Technical
- Rust 2021 Edition
- Async/await com Tokio runtime
- Error handling com `thiserror`
- CLI parsing com `clap` v4
- OAuth2 com `yup-oauth2`

### Known Issues
- Rate limiting não implementado (pode exceder quota da API)
- Uploads são sequenciais (lento para muitos arquivos pequenos)
- Commands `/status` e `/list` são stubs não implementados
- Retry logic configurável mas não utilizado
- Sem lock file (possível race condition com múltiplas instâncias)
- Logs podem vazar tokens em modo debug (`auth.rs:74`)
- Validação de path traversal ausente em ignore patterns

### Documentation
- README com instruções de instalação e uso
- Documentação de configuração em `docs/configuration.md`
- Guia de setup em `docs/setup.md`
- Troubleshooting guide em `docs/troubleshooting.md`

---

## Como Ler Este Changelog

### Tipos de Mudança
- `Added` - Novas features adicionadas
- `Changed` - Mudanças em features existentes
- `Deprecated` - Features que serão removidas em breve
- `Removed` - Features removidas
- `Fixed` - Correções de bugs
- `Security` - Correções de segurança
- `Performance` - Melhorias de performance

### Versionamento Semântico
- **MAJOR** (X.0.0) - Mudanças incompatíveis na API
- **MINOR** (0.X.0) - Novas features compatíveis
- **PATCH** (0.0.X) - Correções de bugs compatíveis

---

## Roadmap Futuro

Veja [ROADMAP.md](ROADMAP.md) para detalhes completos sobre features planejadas.

### V1.0 - Production Ready
- Rate limiting e circuit breaker
- Retry com exponential backoff
- Uploads concorrentes
- Commands `/status` e `/list` implementados
- Testes de integração E2E
- Documentação completa

### V2.0 - Advanced Sync
- Sincronização bidirecional
- Versionamento de arquivos
- Conflito resolution
- Exclusão de arquivos no Drive

### V3.0 - Enterprise
- Dashboard web
- Webhooks do Google Drive
- Suporte a múltiplos clouds
- Métricas e analytics

---

**Mantenedor**: RustDriveSync Contributors
**Licença**: MIT
**Repositório**: https://github.com/usuario/rustdrivesync
