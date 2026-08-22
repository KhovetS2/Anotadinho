---
title: Ciclo 128 — Criar pagina de tipo especifico via paleta de comandos
type: ciclo
ciclo: "128"
status: concluida
date: 2026-08-09
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 128 — Criar pagina de tipo especifico via paleta de comandos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Criar página de tipo específico via paleta de comandos

## Objetivo

Bug reportado: não existe caminho rápido pra criar uma página de tipo
específico (kanban/calendário/tabela/grafo/tags/assets) — hoje é
sempre "criar página em branco → abrir Propriedades → mudar o
dropdown de tipo" (3 passos, 2 deles só por mouse já que o dropdown
não tem atalho dedicado). `api::create_page_with_type` já existe e já
é usado internamente pra "Tags"/"Assets"/"landing", só nunca foi
exposto como comando nomeado na paleta.

## Critérios de aceite

- [x] `PaletteAction` (`command_palette.rs:21-29`) ganha uma variante
      nova, `NewPageOfType(&'static str)` (ou equivalente), e
      `COMMANDS` ganha uma entrada por tipo: "Nova página: Kanban",
      "Nova página: Calendário", "Nova página: Tabela de tarefas",
      "Nova página: Grafo de conexões", junto das já existentes
      ("Ver Tags"/"Ver Assets" continuam sendo os comandos de
      NAVEGAR pra essas páginas fixas, não de criar novas — não
      confundir os dois)
- [x] `app.rs`: handler do novo `PaletteAction` pede o título (mesmo
      `PendingDialog::Prompt` já usado por `new_page_action`) e chama
      `api::create_page_with_type(vault, title, tipo)` — decisão:
      caminho SEPARADO (`prompt_title_and_create_typed`) em vez de
      estender `prompt_title_and_create` (ver Notas)
- [x] Digitar "kanban"/"calendário"/"grafo" etc na paleta filtra e
      acha o comando certo (confirmado ao vivo, ver Notas)
- [x] `cd ui && cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam
- [x] Validação ao vivo via MCP `tauri`: Ctrl+K, digitar "grafo",
      escolher "Nova página: Grafo de conexões", confirmado que a
      página nasce já com `type: graph` no frontmatter (sem passar
      pelo painel de Propriedades) — página de teste removida depois

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Adicionar os mesmos atalhos de criação típada no botão "+" da
  sidebar (que hoje só oferece templates markdown, ciclo 100/113) —
  a paleta de comandos já é o "hub" de ações rápidas por teclado
  deste app; manter os dois fluxos consistentes fica pra outro ciclo
  se o usuário pedir
- Templates PRÉ-PREENCHIDOS por tipo (ex: um kanban já nascendo com
  colunas padrão) — só cria a página vazia do tipo certo, igual
  `create_page_with_type` já faz hoje pros outros tipos

## Notas

Fecha o gap relatado ("no menu não tem [como] criar uma página desse
tipo grafo") da forma mais consistente com o resto do app: a paleta
de comandos já É o lugar onde ações nomeadas por teclado vivem (ciclo
091), então é aí que uma ação nova de criação deve entrar, não um
menu novo.

Decisão de implementação: `prompt_title_and_create_typed` ficou como
função SEPARADA de `prompt_title_and_create`, em vez de generalizar
esta última pra aceitar tanto template quanto page_type — os dois
"resolvem o conteúdo inicial da página" de formas mutuamente
exclusivas (template markdown vs. `type:` de frontmatter), então um
parâmetro unificado (`enum NewPageKind`) só empurraria a decisão pra
dentro de um `match` a mais sem reduzir duplicação de verdade. A
duplicação real entre as duas funções é só o corpo do
`PendingDialog::Prompt` (pedir título) — pequena o bastante pra não
compensar abstrair.

Validação ao vivo teve uma pegadinha de automação (não bug do app):
`Ctrl+K` só abre a paleta se o elemento `.app-root` (que tem o
`onkeydown` do `GlobalKeymap`) estiver focado — depois de um
`location.reload()` o foco vai pro `<body>`, então precisei
`.focus()` no `.app-root` antes de mandar `Ctrl+K` via
`webview_keyboard`. Mesma categoria de detalhe de teste já visto nos
ciclos 123-127 (nada a ver com o comportamento real do app pra um
usuário, cujo clique/tab inicial já bota o foco lá).

Confirmado ao vivo: digitar "grafo" filtrou a lista pra só 3 itens
("Nova página: Grafo de conexões", "grafo" [página existente],
"guia-agent-os" [conteúdo que menciona "grafo"]) — filtro por
label/título/conteúdo funcionando junto, sem comando novo quebrar o
que já existia.

## Resultado

# Ciclo 128 - done

## Resumo

A paleta de comandos (Ctrl+K) ganha 4 comandos novos — "Nova página:
Kanban/Calendário/Tabela de tarefas/Grafo de conexões" — que criam a
página já com o `type:` certo no frontmatter, sem precisar passar pelo
painel de Propriedades. Fecha o gap relatado pelo usuário ("no menu não
tem como criar uma página desse tipo grafo").

## Arquivos criados/modificados

- `ui/src/components/command_palette.rs` — `PaletteAction::NewPageOfType`
  + 4 entradas em `COMMANDS`
- `ui/src/app.rs` — `prompt_title_and_create_typed` (caminho separado
  de `prompt_title_and_create`, ver Notas do arquivo de task) + novo
  braço no `match` de `on_palette_action`

## Testes

`cd ui && cargo test --lib`: 79. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: Ctrl+K → digitar "grafo" → filtrou
pra 3 itens → clicar "Nova página: Grafo de conexões" → modal de
título → "Grafo teste ciclo 128" → OK → arquivo criado com
`type: graph` no frontmatter confirmado direto no disco → `GraphView`
renderizou pra essa página → página de teste removida do vault depois.

## Notas

Ver Notas detalhadas no arquivo de task sobre a decisão de função
separada (vs. generalizar `prompt_title_and_create`) e a pegadinha de
automação do `Ctrl+K` precisar de foco em `.app-root` depois de um
reload (não afeta usuário real).

Próximo: auditoria final de paridade de teclado + atualização do
cheatsheet (129) — último ciclo deste tema.
