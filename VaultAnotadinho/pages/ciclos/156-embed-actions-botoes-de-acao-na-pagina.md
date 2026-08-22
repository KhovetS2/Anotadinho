---
title: "Ciclo 156 — Embed actions: botões de ação na página"
type: ciclo
ciclo: "156"
status: concluida
date: 2026-08-19
prioridade: alta
depende_de: ["148"]
tags:
- ciclo
---

# Ciclo 156 — Embed actions: botões de ação na página

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Embed actions: botões de ação na página

## Objetivo

Todo embed até aqui MOSTRA coisas. Este FAZ coisas: transforma uma
página comum (em especial uma `type: landing`) num painel operável.
No fluxo do agent-os, criar uma spec hoje é: abrir a sidebar, escolher
a pasta certa, escolher o template certo, digitar o título. Com um
botão `new-from-template` isso vira um clique — e o mesmo painel serve
pro humano e pro agente, porque o que o botão faz está declarado em
YAML legível no `.md`.

## Critérios de aceite

- [x] `EmbedKind::Actions` + `{{ type: "actions" }}`
- [x] `ActionsEmbedData { layout: ActionsLayout (Row|Grid), buttons:
      Vec<ActionButton { label, icon, variant, action: ActionSpec }> }`
      com `ActionSpec` enum serde-tagged por `action:`:
      - `new-from-template { template, folder, title_prompt }` →
        `api::create_page_from_template` + navega pra página criada
      - `open-page { path }` → `on_page_selected`
      - `set-property { path, field, value }` → grava frontmatter pelo
        mesmo caminho do painel de propriedades (ciclo 099), sem
        duplicar a lógica de escrita
      - `run-search { query }` → abre a paleta de comandos já filtrada
- [x] Componente `embeds/inline_actions.rs`: botões com ícone
      (`components/icon.rs`) e variante visual (primário/fantasma),
      layout linha ou grade
- [x] PARCIAL: remover botão tem UI (botão "×" no hover de cada um).
      Adicionar/configurar botão pelo modal NÃO entrou — ver Notas: a
      configuração de um botão tem 6 campos que dependem da ação
      escolhida, e o embed já é utilizável escrevendo o YAML (que é o
      caminho do agente e o que o ciclo 160 usa). Virou a **task 163**
- [x] Ação que falha mostra o erro via `PendingDialog::Alert`, sem
      deixar o embed em estado inconsistente
- [x] `set-property` avisa por `Alert` o que gravou (e onde), que é o
      sinal de que a página aberta noutra aba está desatualizada
- [x] Botão é `<button>` de verdade: foco visível, Enter/Espaço
      ativam, `data-nav-item`/`data-nav-group`
- [x] Testes de round-trip de cada variante de `ActionSpec`, incluindo
      uma ação desconhecida (arquivo escrito por versão futura ou por
      um agente) que degrada pra botão desabilitado com aviso, sem
      perder o YAML original

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Rodar comando de shell / invocar agente externo a partir do botão —
  é a superfície de ataque óbvia (um `.md` de terceiro executando
  código ao abrir a página); as ações são um conjunto FECHADO de
  operações do próprio app
- Confirmação/undo de `set-property` além do histórico via git (117)
- Encadear várias ações num botão só

## Notas

`cargo test -p anotadinho-core`: 143 (139 + 4 de actions). `cargo test
--workspace`, `cd ui && cargo test --lib` (26), `trunk build`, `cargo
build --manifest-path src-tauri/Cargo.toml`: OK.

Plumbing necessário: `create_page_from_template` não aceitava pasta de
destino (o handler cravava `None`) — sem isso "Nova spec" cairia em
`pages/` em vez de `pages/specs`. Agora `folder_path: Option<String>`
atravessa ipc → comando Tauri → `api`, com os 3 chamadores existentes
passando `None`. E `run-search` precisou de `on_search` indo de `App`
até o embed (mesmo caminho de `on_page_selected`) + `initial_query` na
paleta; Ctrl+K limpa a consulta ao abrir, pra não reabrir com o termo
anterior.

**Bug pré-existente encontrado:** reserializar frontmatter pelo caminho
tipado grava `created: null`/`updated: null`/`type: null` na página.
Vale pro `anotadinho-cli set-property` (ciclo 116) também. Virou a
**task 162**.

**Escopo não entregue:** modal de configuração de botão (task 163). O
embed funciona escrevendo o YAML, que é o caminho do agente e o que o
ciclo 160 usa.

Validação ao vivo (MCP `tauri`), com uma página de 5 botões: "Nova
spec" pediu o título, criou `pages/specs/spec-de-teste-do-ciclo-156.md`
pelo template (com `{{title}}`/`{{date}}` resolvidos) e navegou pra
ela; "Abrir roadmap" abriu o kanban em aba nova; "Buscar" abriu a
paleta já com `agent` preenchido; "Fechar spec" gravou `status: done`
no arquivo e avisou; o botão com ação `rodar-foguete` (inventada) veio
desabilitado com "Ação desconhecida: rodar-foguete" e o YAML dele
intacto.

A lista fechada de ações é decisão de segurança, não de escopo:
registrar isso na task pra não ser "melhorado" depois sem querer.

## Resultado

# Ciclo 156 - done

## Resumo

`{{ type: "actions" }}` — o embed que FAZ coisas. Os outros mostram;
este opera o vault: cria página a partir de template na pasta certa,
abre página, grava propriedade de frontmatter e abre a busca. É o que
transforma uma `type: landing` num painel operável — pelo humano no
clique e pelo agente lendo o mesmo YAML.

Lista de ações FECHADA por decisão de segurança: um `.md` de terceiro
não pode executar shell só por ser aberto. Ação desconhecida vira botão
desabilitado com aviso, preservando o YAML.

## Arquivos criados/modificados

- `crates/core/src/embed.rs` — `ActionsLayout`, `ActionSpec`,
  `ActionButton`, `ActionsEmbedData` + 4 testes
- `crates/core/src/index.rs` — braço do novo tipo
- `ui/src/components/embeds/inline_actions.rs` (novo)
- `ui/src/components/embeds/mod.rs` — registro, dispatcher, prop
  `on_search`
- `crates/ipc`, `src-tauri`, `ui/src/api.rs` — `folder_path` opcional
  em `create_page_from_template`
- `ui/src/{app,components/page_view,components/editor}.rs` — `on_search`
  de ponta a ponta
- `ui/src/components/command_palette.rs` — prop `initial_query`
- `ui/src/styles/main.css` — `.actions-embed*`

## Testes adicionados

- round-trip com as 4 ações e layout em grade
- resolução de cada nome de ação nos campos certos
- ação desconhecida vira `Unknown`, não é clicável e NÃO perde o YAML
- botão incompleto (sem template/path) não é clicável

## Problemas encontrados

- `create_page_from_template` cravava `None` na pasta no handler de
  ipc: "Nova spec" cairia em `pages/` em vez de `pages/specs`. Pasta
  plumada até a API (3 chamadores existentes passam `None`).
- **Pré-existente:** reserializar frontmatter pelo caminho tipado grava
  `created: null`/`updated: null`/`type: null`. Vale pro CLI também →
  **task 162**.
- **Escopo não entregue:** modal de configuração de botão → **task
  163**. O embed é utilizável pelo YAML, que é o caminho do agente e o
  que o ciclo 160 usa.

## Notas para próximos ciclos

- 160 (painel) tem tudo que precisa: callout, columns, query, timeline
  e actions.
- Restam 157/158 (CLI), 159 (toolbar), 160 (painel), mais as tasks de
  bug 161/162 e a 163.
