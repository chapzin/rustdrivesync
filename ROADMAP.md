# RustDriveSync Development Roadmap

## 📋 Índice
- [Status do Projeto](#-status-do-projeto)
- [Sprints Planejados](#-sprints-planejados)
- [Backlog por Categoria](#-backlog-por-categoria)
- [Debt Técnica](#-debt-técnica)
- [Visão de Longo Prazo](#-visão-de-longo-prazo)
- [Métricas de Sucesso](#-métricas-de-sucesso)
- [Ferramentas e Infra](#-ferramentas-e-infra)
- [Próximos Passos](#-próximos-passos-imediatos)

## 📊 Status do Projeto

### Última Atualização: 2026-01-06

#### ✅ Concluído Hoje
- [x] Code review completo
- [x] MIME type seguro com `mime_guess`
- [x] Eliminação de `.unwrap()` em produção
- [x] **Streaming de uploads** (memória constante ~5MB)
- [x] Cálculo incremental de MD5

#### 🎯 Métricas Atuais
- Linhas de código: 3.354 (src/)
- Testes: 25 (100% passing)
- Coverage: ~60% (estimado)
- Arquivos: 43 Rust files
- TODOs conhecidos: 8

#### 🚧 Estado das Features

| Feature | Status | Notas |
|---------|--------|-------|
| Sync unidirecional | ✅ Completo | Produção-ready |
| File watcher | ✅ Completo | Debouncing 2s |
| Streaming uploads | ✅ Completo | Implementado hoje |
| Rate limiting | ❌ Ausente | **BLOQUEADOR P0** |
| Retry logic | ⚠️ Parcial | Config existe, não usado |
| Uploads concorrentes | ❌ Ausente | Performance P1 |
| /status command | ❌ Stub | UX P2 |
| /list command | ❌ Stub | UX P2 |
| Sync bidirecional | ❌ Futuro | V2 feature |

---

## 🚀 Sprints Planejados

### Sprint 1: Estabilidade de Produção (8-10h)
**Objetivo**: Tornar o projeto verdadeiramente production-ready

**Tarefas**:
1. [ ] **Rate Limiting** (2h)
   - Adicionar `governor` crate
   - Implementar `RateLimiter` com 3 req/s
   - Integrar em `DriveClient`
   - Testes unitários
   - **Arquivo**: `src/google_drive/client.rs`

2. [ ] **Retry com Exponential Backoff** (2h)
   - Usar `RetryConfig` existente
   - Implementar backoff: 1s, 2s, 4s, 8s
   - Adicionar em `DriveUploader` e `DriveClient`
   - Logar tentativas
   - **Arquivo**: `src/google_drive/upload.rs`

3. [ ] **Circuit Breaker** (2h)
   - Detectar 5 falhas consecutivas
   - Abrir circuito por 30s
   - Retry gradual
   - Métricas de saúde
   - **Arquivo**: `src/google_drive/client.rs`

4. [ ] **Tratamento de Quota Exceeded** (1h)
   - Detectar erro 403 quota
   - Pausar sync por 1h
   - Notificar usuário
   - Logs claros
   - **Arquivo**: `src/google_drive/client.rs`

5. [ ] **Fix Token Leakage** (1h)
   - Sanitizar logs em `auth.rs:74`
   - Adicionar redação automática
   - Testes de segurança
   - **Arquivo**: `src/google_drive/auth.rs`

**Entregáveis**:
- ✅ Sistema robusto contra API failures
- ✅ Não ultrapassa quota do Google Drive
- ✅ Logs seguros

**Métricas de Sucesso**:
- 0 crashes por rate limit
- 0 token leakage em logs
- Retry automático funciona

---

### Sprint 2: Performance e Qualidade (8-10h)
**Objetivo**: Tornar o sync 10x mais rápido e confiável

**Tarefas**:
1. [ ] **Uploads Concorrentes** (3h)
   - Usar `FuturesUnordered`
   - Paralelismo de 5 uploads simultâneos
   - Progress tracking agregado
   - Testes de concorrência
   - **Arquivo**: `src/sync/engine.rs`

2. [ ] **Validação de Path Traversal** (1h)
   - Verificar patterns de ignore
   - Prevenir `../` attacks
   - Normalizar paths
   - Testes de segurança
   - **Arquivo**: `src/watcher/file_watcher.rs`

3. [ ] **Testes de Integração E2E** (4h)
   - Criar arquivo temp > 100MB
   - Testar upload completo
   - Verificar MD5 remoto
   - Testar watch mode
   - Testar retry logic
   - **Arquivo**: `tests/integration_test.rs` (novo)

4. [ ] **Lock File para State** (1h)
   - Criar `.sync_state.lock`
   - Prevenir 2 instâncias simultâneas
   - Cleanup automático
   - Testes de race condition
   - **Arquivo**: `src/sync/state.rs`

**Entregáveis**:
- ✅ Sync 5-10x mais rápido
- ✅ Testes E2E robustos
- ✅ Sem race conditions

**Métricas de Sucesso**:
- Upload de 100 arquivos < 10s (vs ~50s atual)
- Coverage > 70%
- 0 race conditions

---

### Sprint 3: UX e Documentação (6-8h)
**Objetivo**: Melhorar experiência do usuário

**Tarefas**:
1. [ ] **Implementar /status** (2h)
   - Mostrar arquivos pendentes
   - Última sincronização
   - Erros recentes
   - Estatísticas
   - **Arquivo**: `src/cli/commands.rs`

2. [ ] **Implementar /list** (2h)
   - Listar arquivos locais
   - Listar arquivos remotos
   - Mostrar diff
   - Filtros e busca
   - **Arquivo**: `src/cli/commands.rs`

3. [ ] **Atualizar Documentação** (2h)
   - Corrigir README roadmap
   - Adicionar examples/basic_usage.rs
   - Documentar env vars
   - Tutorial de troubleshooting
   - **Arquivos**: `README.md`, `examples/`

4. [ ] **Remover Stub Files** (1h)
   - Deletar src/core/* (não usado)
   - Limpar TODOs resolvidos
   - Atualizar imports
   - **Arquivo**: `src/core/` (deletar)

**Entregáveis**:
- ✅ Commands básicos funcionais
- ✅ Documentação atualizada
- ✅ Codebase limpo

---

### Sprint 4+: Features Avançadas (Futuro)
- Sincronização bidirecional (20h)
- Dashboard web (40h)
- Webhooks (10h)
- Multi-origem (15h)
- Compressão (5h)
- Criptografia E2E (10h)

---

## 📦 Backlog por Categoria

### 🛡️ Segurança

**P0 - Crítico**
- [ ] Rate limiting
- [ ] Fix token leakage
- [ ] Tratamento de quota exceeded

**P1 - Importante**
- [ ] Path traversal validation
- [ ] Criptografia end-to-end (opcional)
- [ ] Audit logs

**P2 - Melhorias**
- [ ] 2FA para CLI
- [ ] Token rotation automático
- [ ] Detecção de anomalias

---

### ⚡ Performance

**P0 - Crítico**
- [x] Streaming de uploads ✅ FEITO

**P1 - Importante**
- [ ] Uploads concorrentes
- [ ] Delta sync (só mudanças)
- [ ] Compressão antes do upload

**P2 - Melhorias**
- [ ] Cache de metadados do Drive
- [ ] Batch operations
- [ ] HTTP/2 pipelining

---

### 🎨 Features

**MVP - Mínimo Viável**
- [x] Sync unidirecional
- [x] File watcher
- [x] Autenticação OAuth2
- [ ] /status command
- [ ] /list command

**V1.0 - Produção**
- [ ] Rate limiting
- [ ] Retry logic
- [ ] Uploads concorrentes
- [ ] Documentação completa

**V2.0 - Avançado**
- [ ] Sync bidirecional
- [ ] Exclusão de arquivos no Drive
- [ ] Versionamento
- [ ] Conflito resolution

**V3.0 - Enterprise**
- [ ] Dashboard web
- [ ] Webhooks
- [ ] Multi-origem
- [ ] Teams & sharing
- [ ] Métricas e analytics

---

### 📚 Documentação

**P0 - Essencial**
- [x] Criar ROADMAP.md ✅
- [x] Criar CHANGELOG.md ✅
- [ ] Atualizar README roadmap

**P1 - Importante**
- [ ] Tutorial de setup
- [ ] Exemplos de uso
- [ ] Troubleshooting guide

**P2 - Melhorias**
- [ ] API documentation
- [ ] Architecture guide
- [ ] Contributing guide
- [ ] Security policy

---

### 🧪 Testes

**P0 - Crítico**
- [ ] Testes de integração E2E

**P1 - Importante**
- [ ] Testes de performance
- [ ] Testes de segurança
- [ ] Coverage > 80%

**P2 - Melhorias**
- [ ] Chaos engineering
- [ ] Load testing
- [ ] Fuzz testing

---

## 🔧 Debt Técnica

### Crítico (Deve ser resolvido antes de V1.0)

1. **Retry logic não implementado**
   - Config existe em `RetryConfig` mas não é usado
   - Localização: `src/google_drive/upload.rs`
   - Esforço: 2h
   - Risco: Falhas transitórias → uploads perdidos

2. **Estado sem lock file**
   - 2 instâncias simultâneas → corrupção de estado
   - Localização: `src/sync/state.rs`
   - Esforço: 1h
   - Risco: Perda de dados

3. **Stub files não usados**
   - `src/core/*` está vazio
   - Deve ser deletado ou implementado
   - Esforço: 30min

### Importante (Deve ser resolvido antes de V2.0)

1. **Acoplamento forte ao SyncEngine**
   - Dificulta testes e mocking
   - Sugestão: Dependency injection
   - Esforço: 6h

2. **Sem abstração de storage**
   - Acoplado ao Google Drive
   - Dificulta adicionar Dropbox, S3, etc
   - Esforço: 8h

3. **Logs não estruturados**
   - Usa `tracing` mas sem estrutura JSON
   - Dificulta parsing e analytics
   - Esforço: 2h

### Melhorias (Nice to have)

1. **Usar `thiserror` mais amplamente**
   - Alguns erros ainda usam `.map_err`
   - Pode ser simplificado
   - Esforço: 2h

2. **Comandos CLI com subcommands**
   - Usar `clap` subcommands em vez de flags
   - Melhor UX: `rust-sync status` vs `rust-sync --status`
   - Esforço: 3h

---

## 🔮 Visão de Longo Prazo

### V1.0 - Production Ready (2-3 semanas)
**Objetivo**: Software estável, confiável, rápido

**Features**:
- ✅ Sync unidirecional robusto
- ✅ Rate limiting e retry
- ✅ Uploads concorrentes
- ✅ Commands básicos (/status, /list)
- ✅ Documentação completa
- ✅ Testes E2E

**Público-alvo**: Usuários técnicos, desenvolvedores

**Métricas de Lançamento**:
- 0 bugs críticos
- Coverage > 70%
- Documentação 100% completa
- Performance: 100 arquivos em < 10s

---

### V2.0 - Advanced Sync (1-2 meses)
**Objetivo**: Feature parity com Dropbox/Google Backup

**Features**:
- 🔄 Sincronização bidirecional
- 🗑️ Exclusão de arquivos no Drive
- 📜 Versionamento de arquivos
- ⚙️ Conflito resolution (last-write-wins ou manual)
- 🔐 Criptografia end-to-end (opcional)

**Público-alvo**: Power users, pequenas empresas

**Métricas de Lançamento**:
- Sync bidirecional < 10s latência
- 0 perda de dados em conflitos
- Criptografia AES-256

---

### V3.0 - Enterprise Grade (3-6 meses)
**Objetivo**: Solução enterprise com monitoramento

**Features**:
- 📊 Dashboard web (React + WebAssembly backend)
- 🔔 Webhooks do Google Drive (sync instantâneo)
- 👥 Suporte a teams e compartilhamento
- 📈 Métricas e analytics
- 🔌 Plugin system para extensibilidade
- ☁️ Suporte a múltiplos clouds (S3, Dropbox, OneDrive)

**Público-alvo**: Empresas, equipes

**Métricas de Lançamento**:
- Dashboard uptime > 99.9%
- Webhooks < 1s latência
- Suporte a 3+ providers

---

## 📊 Métricas de Sucesso

### Técnicas
- ✅ 0 crashes em produção por > 7 dias
- ✅ Uptime do sync > 99.9%
- ✅ Throughput > 100 MB/s em rede 1Gbps
- ✅ Latência de sync < 5s após mudança
- ✅ Coverage de testes > 80%

### Negócio
- 📈 > 1000 downloads no crates.io
- ⭐ > 100 stars no GitHub
- 🐛 < 5 bugs críticos abertos
- 📚 Documentação completa (100% das features)

### UX
- 😊 NPS > 50
- ⏱️ Tempo de setup < 5 minutos
- 📖 Menos de 3 perguntas frequentes no GitHub issues

---

## 🛠️ Ferramentas e Infra

### Desenvolvimento
- **Rust 2021 Edition** (atual)
- **Tokio async runtime**
- **Clap v4** para CLI
- **Tracing** para logs

### Novas Dependências Necessárias
- `governor` - Rate limiting
- `retry` ou `again` - Retry logic
- `fs2` - Lock files
- `serde_json` estruturado - Logs JSON

### CI/CD (Recomendado)
- **GitHub Actions**
  - Build em Linux/macOS/Windows
  - Testes automatizados
  - Clippy linting
  - Coverage com tarpaulin

### Release Process
1. Atualizar CHANGELOG.md
2. Bump version em Cargo.toml
3. Tag git: `v1.0.0`
4. `cargo publish`
5. GitHub release com binários

---

## 🎯 Próximos Passos Imediatos

### Esta Semana
- [x] Criar arquivo `ROADMAP.md` ✅
- [x] Criar `CHANGELOG.md` ✅
- [ ] Atualizar README.md (seção roadmap)
- [ ] Criar GitHub Project para tracking
- [ ] Criar labels: P0, P1, P2, bug, feature, docs

### Próxima Sessão de Dev
1. [ ] Implementar rate limiting (Sprint 1, tarefa 1)
2. [ ] Implementar retry logic (Sprint 1, tarefa 2)
3. [ ] Adicionar testes de integração básicos

---

## 📝 Notas

### Decisões Arquiteturais
1. **Por que streaming?** - Permite upload de arquivos > RAM disponível
2. **Por que tokio?** - Melhor suporte async em Rust
3. **Por que google-drive3?** - Biblioteca oficial do Google

### Riscos Conhecidos
1. **Google Drive API** - Quota limits podem bloquear usuários
2. **Filesystem events** - Nem todos OS geram eventos consistentes
3. **State corruption** - Sem lock pode haver race conditions

### Alternativas Consideradas
1. **S3 em vez de Google Drive** - Decidido: manter Drive como foco inicial
2. **Polling em vez de file watcher** - Decidido: file watcher mais eficiente
3. **SQLite em vez de JSON para estado** - Futuro: V2.0

---

## 🙏 Contribuindo

Este roadmap é um documento vivo. Contribuições são bem-vindas!

Para sugerir mudanças:
1. Abra uma issue explicando a proposta
2. Referencie este ROADMAP.md
3. Discuta prioridades e esforço
4. Crie PR após aprovação

---

**Última revisão**: 2026-01-06
**Próxima revisão planejada**: 2026-02-06
