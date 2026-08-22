---
title: Ciclo 089 — Landing page novo page_type e definir como inicio
type: ciclo
ciclo: "089"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 089 — Landing page novo page_type e definir como inicio

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Landing page: novo page_type e "definir como início"

## Objetivo

Quarto ciclo do conjunto grande. Página `type: landing` (mesmo Editor de
sempre, customizável via o sistema de embeds inline já existente) +
botão "definir como início" que abre essa página automaticamente ao
abrir o vault — a peça que, junto com wikilinks (ciclo 087) e o
calendário modo Vault (próximo ciclo), permite montar um dashboard
pessoal.

## Critérios de aceite

- [x] `ui/src/components/page_view.rs`: `"landing" | _ =>` cai no mesmo
      `Editor` de sempre — nenhum componente novo, só reconhecimento do
      tipo (o corpo já é customizável via embeds inline, sem mecanismo
      extra)
- [x] Botão "🏠+" no cabeçalho de Pages da sidebar cria uma página
      `type: landing`
- [x] `ui/src/state.rs`: `save_home_page`/`load_home_page`/
      `clear_home_page`, por vault (chave inclui o path do vault)
- [x] Editor ganha botão 🏠 (ativo/inativo por página) que marca/desmarca
      a página atual como início do vault
- [x] `App` abre a página de início automaticamente ao abrir um vault
      (boot com vault salvo OU escolher um vault novo), só se nenhuma
      página já estiver selecionada
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

- Ícone diferente pra páginas `landing` na árvore da sidebar —
  `PageMeta` não carrega `page_type` (só `path`/`title`/`section`),
  adicionar isso tocaria vault/ipc/api de novo; a sidebar continua
  mostrando 📄 pra landing pages também
- Template/scaffold pré-populado (ex: já vir com um embed de calendário
  de exemplo) — a página nasce vazia como qualquer outra, o usuário
  monta do zero com o menu `/`

## Notas

`create_page_with_type` (existia desde os primeiros ciclos, mas sem
nenhum caller de UI pra tipos além de "md" — kanban/calendar/table são
criados manualmente editando frontmatter) ganhou seu primeiro uso real
via `on_new_landing` na sidebar.

"Início" é uma preferência de CLIENTE (localStorage, chaveada por
`vault_path`), não algo salvo no vault — não faz sentido versionar
"qual página abre sozinha" junto com o conteúdo das páginas.

Validado ao vivo via MCP `tauri`: criar página "Minha Home" via 🏠+ →
frontmatter confirma `type: landing`; clicar 🏠 no editor ativa
(confirma `localStorage['anotadinho.home_page::<vault>']`); reload
completo da página (simula reabrir o app) abre "minha-home"
automaticamente sem precisar clicar em nada; clicar 🏠 de novo desmarca.
Página de teste e chave de localStorage removidas antes de fechar o
ciclo.

Achado de metodologia de teste (reafirma nota do ciclo 087): setar
`input.value` + disparar `input` + clicar "OK" do modal de Prompt na
MESMA chamada de `webview_execute_js` corre risco de o clique disparar
antes do estado do Yew (`input_value`) processar o evento de input —
produziu uma página criada com o valor DEFAULT do prompt em vez do
valor digitado. Fix: sempre separar "digitar" e "confirmar" em chamadas
de ferramenta distintas ao testar diálogos de Prompt.

## Resultado

# Ciclo 089 - done

## Resumo

Quarto ciclo do conjunto grande. `type: landing` reusa o `Editor`
normal (nada de componente novo); botão 🏠 no editor marca a página
atual como início do vault (localStorage, por vault); `App` abre a
página de início automaticamente ao abrir o vault.

## Arquivos criados/modificados

- `ui/src/components/page_view.rs` — reconhece `"landing"`
- `ui/src/state.rs` — `save_home_page`/`load_home_page`/`clear_home_page`
- `ui/src/components/editor.rs` — botão 🏠 (`is_home`/`toggle_home`)
- `ui/src/components/sidebar.rs` — botão "🏠+" cria página landing
  (`on_new_landing`)
- `ui/src/app.rs` — efeito que abre a página de início ao abrir o vault
- `ui/src/styles/main.css` — `.editor__home-btn*`

## Testes

Sem testes novos (composição de mecanismos já testados —
`create_page_with_type`, localStorage, dispatch de callback — validado
ao vivo). `cargo test --workspace`: 48. `cd ui && cargo test --lib`: 62.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Criar landing page → frontmatter `type: landing` confirmado; marcar
como início → reload completo abre ela sozinha; desmarcar funciona.
Detalhes no arquivo de task.

## Notas

Próximo: calendário modo Vault (a peça que conecta wikilinks + landing
page ao exemplo do usuário — calendário mostrando tarefas linkadas com
data/hora de entrega).
