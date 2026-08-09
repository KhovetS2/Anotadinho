---
id: "128"
titulo: "Criar pagina de tipo especifico via paleta de comandos"
status: done
criado: 2026-08-09
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
