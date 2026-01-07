# Como Publicar no crates.io

## Pré-requisitos

1. **Conta no crates.io**: Crie uma conta em https://crates.io/
2. **Token de API**: Gere um token em https://crates.io/me
3. **Login via cargo**:
   ```bash
   cargo login <SEU_TOKEN>
   ```

## Checklist Pré-Publicação

- [x] Versão atualizada no `Cargo.toml` (1.0.0)
- [x] Metadados completos (description, keywords, categories, etc.)
- [x] `LICENSE` file presente
- [x] `README.md` presente e atualizado
- [x] `CHANGELOG.md` atualizado com a versão atual
- [x] Todos os testes passando: `cargo test --all-features`
- [x] Build em release funcionando: `cargo build --release`
- [x] Linting sem erros: `cargo clippy -- -D warnings`
- [x] Formatação correta: `cargo fmt -- --check`
- [x] Dry-run bem-sucedido: `cargo publish --dry-run`
- [x] Commit e push de todas as mudanças
- [x] Tag de versão criada: `git tag v1.0.0`

## Publicar

### 1. Verificação Final

```bash
# Test
cargo test --all-features --workspace

# Build release
cargo build --release

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all -- --check

# Dry run
cargo publish --dry-run
```

### 2. Push para GitHub

```bash
# Push commits
git push origin master

# Push tags
git push origin --tags
```

### 3. Publicar no crates.io

```bash
cargo publish
```

**IMPORTANTE**: Após publicar, a versão **NÃO PODE SER DELETADA**. Apenas pode ser "yanked" (marcada como deprecated).

### 4. Verificar Publicação

```bash
# Instalar da versão publicada
cargo install rustdrivesync

# Testar
rustdrivesync --version
```

## Versões Futuras

Para publicar novas versões:

1. Atualizar `version` no `Cargo.toml`
2. Atualizar `CHANGELOG.md`
3. Fazer commit: `git commit -am "chore: bump version to X.Y.Z"`
4. Criar tag: `git tag vX.Y.Z`
5. Push: `git push origin master --tags`
6. Publicar: `cargo publish`

## Política de Versionamento Semântico

- **MAJOR** (X.0.0): Breaking changes (incompatibilidade com versão anterior)
- **MINOR** (0.X.0): Novas features (compatível)
- **PATCH** (0.0.X): Bug fixes (compatível)

Exemplos:
- `1.0.0` → `1.1.0`: Nova feature (backend S3)
- `1.1.0` → `1.1.1`: Bug fix
- `1.x.x` → `2.0.0`: Breaking change (API incompatível)

## Yanking (Deprecar Versão)

Se você publicou uma versão com bug crítico:

```bash
# Yank (deprecar)
cargo yank --vers 1.0.0

# Unyank (reverter deprecação)
cargo yank --vers 1.0.0 --undo
```

**Nota**: Yank não remove o pacote, apenas impede novos projetos de usá-lo.

## Links Úteis

- **crates.io**: https://crates.io/crates/rustdrivesync
- **docs.rs**: https://docs.rs/rustdrivesync
- **Cargo Book**: https://doc.rust-lang.org/cargo/reference/publishing.html

## Status Atual

- [x] **V1.0.0**: Pronto para publicação
- [x] Dry-run: ✅ Passou
- [x] GitHub Release: ✅ Criada
- [x] Workflows CI/CD: ✅ Configurados
- [ ] **Publicado no crates.io**: ⏳ Aguardando comando

---

**Para publicar agora**:
```bash
cargo publish
```
