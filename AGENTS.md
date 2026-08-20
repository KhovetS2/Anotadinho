# AGENTS.md — Anotadinho

## Ciclo de desenvolvimento

Cada feature é implementada em UM ciclo. Após cada ciclo:
1. Marcar task como `done` e atualizar checkboxes `[x]`
2. Criar arquivo de status em `cycles/status/{id}-{timestamp}-done.md`
3. Rodar `cargo test --workspace`, `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
4. Commitar com prefixo `feat({id}):` ou `fix({id}):`
5. Push automático ao final

**NÃO perguntar se deve continuar para o próximo ciclo.** Executar direto.

## Stack

| Camada | Tecnologia |
|---|---|
| Runtime | Tauri 2.x |
| Backend | Rust puro (workspace: crates/) |
| UI | Yew 0.21 + WASM + trunk |
| Editor | contenteditable + toolbar + slash commands |
| CSS | BEM, dark theme, tokens em :root |
| Testes | `cargo test --workspace` (31 testes) |

## Estrutura de diretórios

```
crates/core/   — modelos (Block, Page, Property, MarkdownCodec)
crates/vault/  — I/O (VaultIo, VaultWatcher)
crates/ipc/    — handlers de comandos Tauri
ui/src/        — frontend Yew (components, api, state, styles)
src-tauri/     — entry point Tauri, comandos
cycles/        — tasks, status, failures
docs/          — architecture, design-system, etc
```

## Validação por ciclo

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```

Ciclo que mexe em UI roda também o harness (ciclo 177) — os bugs de
DOM não aparecem no `cargo test`:

```bash
./scripts/dev.sh              # num terminal, deixa o app de pé
node scripts/uitest/run.mjs   # noutro; sai != 0 se algo quebrou
```

Regressão nova encontrada ao validar vira cenário em
`scripts/uitest/cenarios.mjs`, junto do número do ciclo onde apareceu.

## Design system

Ver `docs/design-system.md` para tokens, convenções BEM e templates.

## Serviços externos

- Mermaid.js via CDN em `ui/index.html`
- Playwright MCP para testes E2E (config em `.opencode/opencode.json`)
