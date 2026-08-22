---
title: Ciclo 088 — Backlinks painel on-demand no editor
type: ciclo
ciclo: "088"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: ["087"]
tags:
- ciclo
---

# Ciclo 088 — Backlinks painel on-demand no editor

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Backlinks: painel on-demand no editor

## Objetivo

Terceiro ciclo do conjunto grande. Painel colapsável no fim do editor
mostrando quais páginas linkam pra página atual via `[[Título]]`
(depende do ciclo 087).

## Critérios de aceite

- [x] `ui/src/components/editor.rs`: estado `backlinks` + efeito
      disparado por troca de página, reaproveitando `api::search_content`
      com a query `"[[Título]]"` literal (evita expor uma rota IPC nova
      só pra isso)
- [x] Resultado filtra a própria página (evita listar a página como
      backlink de si mesma se ela citar o próprio título)
- [x] Painel (`<details>`, mesmo padrão das pastas na sidebar) só
      aparece quando há pelo menos 1 backlink; clicar um item navega
      pra página de origem
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

- Índice persistido/mantido incrementalmente — calculado sob demanda ao
  abrir a página, mesmo perfil de custo da busca ingênua já existente;
  pode reusar o índice de busca real (ciclo #83, task tracker) se algum
  dia isso ficar lento de verdade
- Melhorar a qualidade do excerpt (hoje é "N chars ao redor do match",
  herdado de `search_content` — pode incluir frontmatter em arquivos
  curtos) — fica pro ciclo de busca full-text de verdade

## Notas

Reaproveitou `api::search_content(vault_path, "[[Título]]")` em vez de
criar uma rota nova — o "grep" que já existe no vault já faz
exatamente o que backlinks precisa (achar páginas com esse substring
literal). Menos código novo, mesma limitação (excerpt curto, sem
ranking) que a busca em si já tinha, resolvida quando #83 (busca
full-text real) acontecer.

Validado ao vivo via MCP `tauri`: página `teste` com `[[kanban]]` no
corpo, salva; abrir a página `kanban` mostra "🔗 Backlinks (1)",
expandir mostra o item "teste" com excerpt, clicar navega de volta pra
`teste`. Página sem nenhum backlink (`sobre`) não mostra o painel.
Mudança de teste revertida em `VaultAnotadinho/pages/teste.md`.

## Resultado

# Ciclo 088 - done

## Resumo

Terceiro ciclo do conjunto grande. Painel "Backlinks" colapsável no fim
do editor, calculado sob demanda via `search_content("[[Título]]")`.

## Arquivos criados/modificados

- `ui/src/components/editor.rs` — estado `backlinks`, efeito de busca,
  renderização do painel
- `ui/src/styles/main.css` — `.editor__backlinks*`

## Testes

Sem testes novos (é composição de `search_content`, já testado
indiretamente; UI pura validada ao vivo). `cargo test --workspace`:
48 passaram. `cd ui && cargo test --lib`: 62 passaram.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Página com `[[kanban]]` no corpo → abrir `kanban` mostra "🔗 Backlinks
(1)" → expandir → clicar navega de volta. Página sem backlinks não
mostra o painel. Detalhes no arquivo de task.

## Notas

Próximo: landing page (novo `page_type` + "definir como início").
