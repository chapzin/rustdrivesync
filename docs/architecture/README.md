# RustDriveSync - Documentação de Arquitetura

## Índice

1. [Visão Geral](#visão-geral)
2. [Decisões Arquiteturais (ADRs)](#decisões-arquiteturais-adrs)
3. [Diagramas C4](#diagramas-c4)
4. [Componentes e Módulos](#componentes-e-módulos)
5. [Arquitetura de Dados](#arquitetura-de-dados)
6. [Segurança](#segurança)
7. [Qualidade e Atributos](#qualidade-e-atributos)

## Visão Geral

**RustDriveSync** é uma ferramenta de sincronização de arquivos para Google Drive escrita em Rust, projetada para ser robusta, eficiente e extensível.

### Propósito
- Sincronizar arquivos locais com Google Drive de forma confiável
- Suportar múltiplos modos de operação (única vez, monitoramento contínuo)
- Permitir extensão para outros backends de armazenamento (S3, Dropbox)

### Princípios Arquiteturais
1. **Modularidade**: Separação clara de responsabilidades
2. **Extensibilidade**: Suporte a múltiplos backends via abstrações
3. **Resiliência**: Retry automático, rate limiting, gerenciamento de estado
4. **Performance**: Uploads paralelos, processamento assíncrono
5. **Observabilidade**: Logging estruturado, métricas de progresso

### Tecnologias Core
- **Linguagem**: Rust 1.70+
- **Runtime Assíncrono**: Tokio
- **HTTP Client**: Reqwest
- **Serialização**: Serde
- **Logging**: Tracing
- **Configuração**: TOML

## Estrutura de Documentação

```
docs/architecture/
├── README.md                          # Este arquivo
├── adr/                               # Architecture Decision Records
│   ├── 0001-dependency-injection.md
│   ├── 0002-rate-limiting.md
│   ├── 0003-retry-strategy.md
│   ├── 0004-parallel-uploads.md
│   └── template.md
├── diagrams/                          # Diagramas (Mermaid/PlantUML)
│   ├── c4-context.md
│   ├── c4-container.md
│   ├── c4-component.md
│   ├── data-flow.md
│   ├── deployment.md
│   └── security.md
├── components/                        # Documentação de componentes
│   ├── storage-backend.md
│   ├── sync-engine.md
│   ├── rate-limiter.md
│   └── retry-mechanism.md
├── quality-attributes.md              # Atributos de qualidade
├── security-architecture.md           # Arquitetura de segurança
└── evolution-roadmap.md               # Roadmap de evolução
```

## Navegação Rápida

### Para Desenvolvedores
- [Guia de Componentes](components/)
- [Decisões Arquiteturais](adr/)
- [Padrões de Código](../CONTRIBUTING.md)

### Para Arquitetos
- [Diagramas C4](diagrams/)
- [Atributos de Qualidade](quality-attributes.md)
- [Roadmap de Evolução](evolution-roadmap.md)

### Para Segurança/Compliance
- [Arquitetura de Segurança](security-architecture.md)
- [Modelo de Ameaças](security-architecture.md#modelo-de-ameaças)

## Status da Documentação

| Documento | Status | Última Atualização |
|-----------|--------|-------------------|
| ADR-0001 Dependency Injection | ✅ Completo | 2026-01-07 |
| ADR-0002 Rate Limiting | ✅ Completo | 2026-01-07 |
| ADR-0003 Retry Strategy | ✅ Completo | 2026-01-07 |
| ADR-0004 Parallel Uploads | ✅ Completo | 2026-01-07 |
| Diagrama C4 - Context | ✅ Completo | 2026-01-07 |
| Diagrama C4 - Container | ✅ Completo | 2026-01-07 |
| Diagrama C4 - Component | ✅ Completo | 2026-01-07 |
| Data Flow Diagram | ✅ Completo | 2026-01-07 |
| Security Architecture | ✅ Completo | 2026-01-07 |
| Quality Attributes | ✅ Completo | 2026-01-07 |

## Convenções

### Diagramas
- Usamos **Mermaid** para diagramas versionáveis no Git
- Exportamos PNG/SVG para apresentações
- Mantemos sincronização entre código e diagramas

### ADRs
- Formato Markdown padrão
- Numeração sequencial (0001, 0002, ...)
- Status: Proposed → Accepted → Deprecated → Superseded
- Imutáveis após aceitos (novos ADRs substituem antigos)

### Versionamento
- Documentação versionada junto com código
- Tags Git correspondem a releases
- Changelog de arquitetura em cada release
