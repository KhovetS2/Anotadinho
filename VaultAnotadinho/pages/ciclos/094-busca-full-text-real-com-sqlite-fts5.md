---
title: Ciclo 094 — Busca full-text real com SQLite FTS5
type: ciclo
ciclo: "094"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 094 — Busca full-text real com SQLite FTS5

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Busca full-text real (SQLite FTS5)

## Objetivo

Nono ciclo do conjunto grande. Substitui `VaultIo::search_content`
(scanner ingênuo, só a primeira ocorrência por página, sem ranking) por
um índice de verdade — descoberto durante a implementação que
`rusqlite` com feature `bundled` (que já compila com
`-DSQLITE_ENABLE_FTS5`) **já era uma dependência declarada** desde o
início do projeto (`crates/search/Cargo.toml`), só nunca usada —
decisão original do plano (índice em memória hand-rolled, evitando
SQLite) revista: usar FTS5 de verdade é MENOS código e melhor
qualidade, já que a dependência já estava paga.

## Critérios de aceite

- [x] `crates/search/src/fulltext.rs`: `SearchIndex` real —
      `CREATE VIRTUAL TABLE ... USING fts5`, `index_page`, `search`
      (ranking BM25, `snippet()` com highlight `**termo**`, prefix-match
      por palavra, termos unidos por `OR`, escapados contra injeção de
      operador FTS5) — com 8 testes
- [x] `crates/ipc/src/lib.rs`: `handle_search_content` reconstrói o
      índice a cada busca (lê todas as páginas, indexa, consulta) —
      mesmo custo de I/O do scanner antigo, qualidade de resultado muito
      melhor
- [x] `crates/vault/src/io.rs`: `VaultIo::search_content` removido (só
      tinha 1 caller, que virou o novo `handle_search_content`) — sem
      isso ficaria mantendo 2 implementações de busca em paralelo
- [x] `ui/src/components/sidebar.rs`: `render_excerpt_highlight`
      converte os marcadores `**termo**` do snippet em `<strong>` de
      verdade na lista de resultados
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Índice persistido/mantido incrementalmente entre buscas — reconstruído
  do zero a cada chamada (mesmo perfil de I/O do scanner que substitui);
  otimizar isso é ciclo futuro se o vault crescer o bastante pra doer
- Busca AND estrita (exigir todas as palavras) — usa `OR` entre termos,
  mais tolerante a erro de digitação/lembrar só parte do texto
- Ranking por embeddings/semântica — só léxico (BM25) nesta v1

## Notas

Achado de arquitetura: `rusqlite = { version = "0.31", features =
["bundled"] }` já estava em `crates/search/Cargo.toml` desde cedo no
projeto, mas nunca importado em código nenhum — o `cargo build
--workspace` já vinha compilando essa dependência (SQLite embarcado, C
compilado do zero) sem usar nada dela. Confirmar isso mudou a decisão
registrada no plano original (`jaunty-tinkering-beaver.md`, que dizia
"não uso SQLite/FTS5 — dependência nova desnecessária") — não é mais
uma dependência NOVA, então a lógica de evitar não se aplica.

Bug real pego DURANTE a validação ao vivo (ambiente, não lógica): o
processo Tauri (`cargo-tauri tauri dev`) já estava rodando de ciclos
anteriores com o binário ANTIGO — mudanças em `crates/*` (fora de
`src-tauri/`) não disparam rebuild+restart automático do processo já
rodando (só o watcher de `src-tauri/src/` faz isso; `trunk serve`
recarrega o FRONTEND via reload, mas isso nunca reinicia o processo de
BACKEND). Resultado: testar a busca nova via MCP inicialmente pareceu
"funcionar" mas sem os marcadores de highlight — porque estava batendo
no binário velho. Diagnosticado comparando o timestamp de início do
processo (`ps -o lstart`) com o timestamp dos arquivos editados; fix:
matar `cargo-tauri`/`trunk serve` e relançar via `./scripts/dev.sh`.
Registrado aqui porque é a segunda vez nesta sessão de ciclos que esse
tipo de dessincronia aparece (a primeira foi puramente sobre hot-reload
do frontend, ciclo 086) — vale lembrar que mudanças em `crates/` sempre
exigem reiniciar o processo Tauri inteiro, não só recarregar a página.

Validado ao vivo via MCP `tauri` (depois do restart): busca por
"kanban" retorna 4 páginas com trechos `**kanban**` destacados
corretamente (confirmado via `window.__TAURI_INTERNALS__.invoke`
direto E via a UI da sidebar, com o highlight virando `<strong>` de
verdade depois do fix em `sidebar.rs`).

## Resultado

# Ciclo 094 - done

## Resumo

Nono ciclo do conjunto grande. `crates/search`'s `SearchIndex` vira
real (SQLite FTS5 — `rusqlite` já era dependência declarada, nunca
usada), substituindo o scanner ingênuo de `VaultIo::search_content`
(removido).

## Arquivos criados/modificados

- `crates/search/src/fulltext.rs` — implementação real, 8 testes
- `crates/search/src/lib.rs` — doc atualizado
- `crates/ipc/src/lib.rs` — `handle_search_content` usa `SearchIndex`
- `crates/vault/src/io.rs` — remove `search_content` (só tinha 1 caller)
- `ui/src/components/sidebar.rs` — `render_excerpt_highlight`

## Testes

`cargo test --workspace`: 56 (48 + 8 novos). `cd ui && cargo test
--lib`: 66. Total 122 combinando os dois runs.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Busca por "kanban" retorna 4 páginas com highlight `**termo**` →
`<strong>` de verdade na sidebar. Detalhes no arquivo de task.

## Notas

Bug de ambiente pego e documentado: processo Tauri precisa reiniciar
inteiro (não só reload de página) quando `crates/*` muda, diferente do
frontend WASM que recarrega sozinho. Ver arquivo de task.

Próximo: undo/redo genérico, depois gestão de assets (últimos 2 do
conjunto grande).
