---
title: "Ciclo 086 — Pastas: subdiretorios reais e arvore na sidebar"
type: ciclo
ciclo: "086"
status: concluida
date: 2026-08-07
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 086 — Pastas: subdiretorios reais e arvore na sidebar

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Pastas: subdiretórios reais e árvore na sidebar

## Objetivo

Primeiro ciclo de um conjunto grande pedido pelo usuário (pastas,
wikilinks+backlinks, landing page, paleta de comandos, vim mode, página
de tags, busca full-text, undo/redo, gestão de assets — ver
`/home/elis/.claude/plans/jaunty-tinkering-beaver.md`). Este ciclo:
organização de páginas em pastas. `list_pages()` já suportava
subdiretórios a nível de I/O — o gap era 100% de UI (sidebar em lista
flat) + duas operações novas no `VaultIo` (criar pasta, mover página).

## Critérios de aceite

- [x] `VaultIo::create_folder`/`list_folders`/`move_page`/
      `create_page_in_folder` em `crates/vault/src/io.rs`, com testes
      (idempotência, rejeição de path traversal, pastas vazias visíveis)
- [x] Handlers IPC + comandos Tauri + funções `ui/src/api.rs`
      correspondentes
- [x] Sidebar (`ui/src/components/sidebar.rs`) reconstrói `Pages` como
      árvore (`<details>` nativo por pasta, aberto por padrão) a partir
      de `PageMeta::path` + `list_folders()`; busca ativa volta pra
      lista flat (mais fácil escanear resultados espalhados)
- [x] Botão "📁+" no cabeçalho de Pages cria pasta; cada pasta tem seu
      próprio "+" pra criar página já dentro dela; cada página tem um
      botão "📁" (aparece no hover) pra mover pra outra pasta
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

- Pastas aninhadas criadas pela UI (só o botão raiz "Nova pasta" +
  criar/mover página pra dentro de uma pasta existente) — dá pra chegar
  em subpastas mais profundas movendo manualmente o path, mas não há
  botão "nova subpasta dentro de pasta" nesta v1
- Drag-and-drop de página pra pasta (mover usa um prompt de texto por
  enquanto, não arrastar)
- Pastas dentro de `journals/` — journals continuam flat por data
- Consolidar as 3 cópias duplicadas de `PageMeta`

## Notas

Descoberta na exploração inicial (Explore agent): `list_pages()`
(`crates/vault/src/io.rs:44`, `WalkDir::new(&dir).max_depth(3)`) já
listava arquivos dentro de subdiretórios corretamente — só nunca tinha
UI que criasse ou navegasse essa estrutura. `list_folders()` foi
adicionado porque pastas vazias (criadas mas sem página dentro ainda)
não apareceriam de outra forma, já que `list_pages` só enxerga arquivos.

Descoberta durante a validação ao vivo (ambiente de teste, não bug de
código): a sessão MCP `tauri` conectada não recebeu o hot-reload do
`trunk serve` automaticamente — precisou de `location.reload()`
explícito pra pegar o bundle novo. Também descoberto: o bridge
`webview_execute_js` só aceita scripts de **uma expressão só**
(`function(){...}` com múltiplos statements ou declarações soltas de
`const`/`let` seguidas de outra statement retornam `null` silenciosamente)
— funciona com `(() => { ...; return X; })()` ou uma cadeia de métodos
numa expressão só. Registrado aqui pra não perder tempo de novo em
ciclos futuros que usem o mesmo MCP.

Validado ao vivo via MCP `tauri`: criar pasta "Trabalho" → aparece na
árvore; criar página dentro da pasta via o "+" da pasta → aparece
aninhada; mover a página "sobre" pra dentro de "Trabalho" via o botão
"📁" do item → some da raiz, aparece dentro da pasta, sem duplicar;
clicar a página movida abre o editor normalmente; reload da página
confirma que pasta + páginas persistem em disco (não é só estado de
cliente). Mudanças de teste revertidas em `VaultAnotadinho/` antes de
fechar o ciclo (`git status --short VaultAnotadinho/` limpo).

## Resultado

# Ciclo 086 - done

## Resumo

Primeiro de uma série grande de ciclos (ver plano em
`/home/elis/.claude/plans/jaunty-tinkering-beaver.md`). Adiciona pastas
de verdade (subdiretórios sob `pages/`) e reconstrói a seção Pages da
sidebar como árvore, no lugar de lista flat.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `create_folder`, `list_folders`,
  `move_page`, `create_page_in_folder`, `validate_relative_path`
  (extraído, compartilhado com `resolve_safe_for_write`), 10 testes novos
- `crates/ipc/src/lib.rs` — 4 handlers novos
- `src-tauri/src/main.rs` — 4 comandos Tauri novos
- `ui/src/api.rs` — 4 funções de ponte novas
- `ui/src/components/sidebar.rs` — árvore de pastas (`build_tree`,
  `render_tree`, `render_movable_list`), botões de criar
  pasta/página-na-pasta/mover
- `ui/src/styles/main.css` — `.sidebar-folder*`, `.sidebar-item__move`

## Testes

`cargo test --workspace`: 75 passaram (26 em `anotadinho-vault`, +10
novos). `cd ui && cargo test --lib`: 54 passaram (sem testes novos —
árvore é view somente-leitura sem lógica de parsing nova).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Criar pasta, criar página dentro dela, mover página existente pra
dentro, clicar a página movida (abre normal), reload confirma
persistência em disco. Detalhes no arquivo de task.

## Notas

Nada pendente deste ciclo. Próximo: wikilinks (`[[Título]]` + render +
popup no editor).
