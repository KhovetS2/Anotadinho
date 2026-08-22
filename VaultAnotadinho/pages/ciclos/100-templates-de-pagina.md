---
title: Ciclo 100 — Templates de pagina
type: ciclo
ciclo: "100"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: ["098"]
tags:
- ciclo
---

# Ciclo 100 — Templates de pagina

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Templates de página

## Objetivo

Terceiro ciclo do tema "agent-os readiness" — o item mais pedido pra
esse uso (specs/decisões/registros seguindo uma estrutura repetida).
Templates são páginas normais guardadas em `templates/` na raiz do
vault (pasta visível, não oculta — o usuário edita/cria os próprios
templates como qualquer outra página); criar página a partir de um
template copia o corpo + frontmatter (incluindo `extra`, ciclo 098),
substituindo `{{title}}` pelo título escolhido.

## Critérios de aceite

- [x] `crates/vault/src/io.rs`: `create_page_from_template(template_path,
      title, folder) -> Result<PageMeta>` — lê o template, substitui
      `{{title}}` (corpo E frontmatter) pelo título digitado, escreve a
      página nova (mesma lógica de slug único de `create_page_in`)
- [x] `list_templates() -> Result<Vec<PageMeta>>` — lista arquivos em
      `templates/` (mesmo padrão de `list_pages`, mas escaneando só essa
      pasta)
- [x] Fluxo de "Nova página" (Ctrl+N/paleta) ganha uma etapa opcional de
      escolher um template (lista vazia = comportamento atual,
      inalterado, se `templates/` não existir ou estiver vazia)
- [x] `VaultAnotadinho/templates/` ganha 2-3 exemplos reais (ex: "Spec",
      "Decisão", "Nota de reunião") documentando o recurso pro próprio
      vault de demonstração
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

- Placeholders além de `{{title}}` (ex: `{{date}}`, variáveis
  customizadas) — v1 é só o título, que já cobre o caso mais comum;
  expandir os placeholders é um ciclo futuro se pedirem
- Templates por tipo de página com lógica especial (kanban/calendar já
  têm seu próprio jeito de nascer preenchidos) — templates aqui são só
  pra páginas markdown normais (`type: md` ou sem type)
- Editor visual de template — é um arquivo `.md` normal, editado como
  qualquer página

## Notas

Depende do ciclo 098 (`Frontmatter.extra`) pra templates poderem trazer
propriedades customizadas (ex: um template de "Spec" já vindo com
`status: draft`) sem perdê-las ao copiar.

`templates/` fica FORA de `pages/`/`journals/` — não deveria aparecer
na lista normal de páginas da sidebar nem ser contada como "página" em
lugar nenhum (tags, busca, backlinks); confirmar que `list_pages()`
(que só varre `pages/`/`journals/`) já ignora `templates/` por
construção (deveria, já que só escaneia essas duas pastas fixas).

Confirmado com teste (`list_templates_does_not_leak_into_list_pages`).

Novo `PendingDialog::Select` genérico (`ui/src/dialog.rs` +
`dialog_host.rs`) — modal de lista de opções, encadeia com `Prompt`
igual o padrão já usado (dismiss antes de emitir). Reusável por outros
fluxos futuros que precisem de uma escolha simples antes de um prompt.

O botão "+" da sidebar (`Sidebar::on_new_page`) tem seu PRÓPRIO fluxo
de criação de página, independente de `new_page_action` em `app.rs` —
não foi tocado neste ciclo, conforme o escopo do critério de aceite
("Ctrl+N/paleta"). Extender templates pro botão da sidebar fica pra um
ciclo futuro se pedirem.

Validado ao vivo via MCP `tauri`: Ctrl+N → modal de templates lista os
3 exemplos reais → escolher "spec" → prompt de título → página criada
em `pages/` com `{{title}}` substituído no corpo E no frontmatter,
demais propriedades do template (`status: draft`) preservadas. Testado
também "Página em branco" (comportamento antigo, inalterado).

Bug de descoberta durante a validação (não deste ciclo, ambiente de
teste): atalhos de teclado (`Ctrl+N`/`Ctrl+K`) só disparam se o
`<div class="app-root" tabindex="0">` estiver focado — não é um
listener global em `document`. Isso é esperado (por design, escuta só
o elemento raiz do app), só documentando pra próxima sessão de
validação via MCP não perder tempo re-descobrindo.

## Resultado

# Ciclo 100 - done

## Resumo

Terceiro ciclo do tema "agent-os readiness". Templates de página:
`templates/` na raiz do vault guarda páginas normais que servem de
molde; criar página a partir de um template copia corpo + frontmatter
(incluindo `extra` do ciclo 098), substituindo `{{title}}`. Fluxo de
"Nova página" (Ctrl+N/paleta) ganha uma etapa opcional de escolha de
template via um novo diálogo genérico `PendingDialog::Select`.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `list_templates`, `create_page_from_template`,
  `find_unique_relative_path` (extraído de `create_page_in` pra
  reaproveitar a lógica de slug único), 7 testes novos
- `crates/ipc/src/lib.rs` — `handle_list_templates`,
  `handle_create_page_from_template`
- `src-tauri/src/main.rs` — comandos `list_templates`,
  `create_page_from_template` registrados
- `ui/src/api.rs` — wrappers `list_templates`, `create_page_from_template`
- `ui/src/dialog.rs` — `PendingDialog::Select` (novo, genérico)
- `ui/src/components/dialog_host.rs` — renderiza `Select`
- `ui/src/styles/components.css` — `.modal__select-list`/`.modal__select-item`
- `ui/src/app.rs` — `new_page_action` ganha etapa de escolha de template
  (lista vazia = comportamento antigo, inalterado)
- `VaultAnotadinho/templates/` — 3 exemplos reais: `spec.md`,
  `decisao.md`, `nota-de-reuniao.md`

## Testes

`cargo test --workspace`: 71 (25 core + 1 ipc + 8 search + 37 vault).
`cd ui && cargo test --lib`: 66. Total 137.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: fluxo completo Ctrl+N → escolher
template "spec" → título → página criada com substituição correta em
corpo e frontmatter; "Página em branco" testado também (inalterado).

## Notas

Botão "+" da sidebar tem fluxo próprio de criação, não tocado (fora do
escopo — critério de aceite menciona só Ctrl+N/paleta). Detalhes no
arquivo de task.

Próximo: exportação em massa / dump de contexto (101), pra permitir
levar o vault inteiro (ou um recorte) pro contexto de um agente.
