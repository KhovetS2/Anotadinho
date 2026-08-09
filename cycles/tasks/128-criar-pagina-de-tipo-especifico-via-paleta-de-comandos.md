---
id: "128"
titulo: "Criar pagina de tipo especifico via paleta de comandos"
status: pending
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

- [ ] `PaletteAction` (`command_palette.rs:21-29`) ganha uma variante
      nova, `NewPageOfType(&'static str)` (ou equivalente), e
      `COMMANDS` ganha uma entrada por tipo: "Nova página: Kanban",
      "Nova página: Calendário", "Nova página: Tabela de tarefas",
      "Nova página: Grafo de conexões", junto das já existentes
      ("Ver Tags"/"Ver Assets" continuam sendo os comandos de
      NAVEGAR pra essas páginas fixas, não de criar novas — não
      confundir os dois)
- [ ] `app.rs`: handler do novo `PaletteAction` pede o título (mesmo
      `PendingDialog::Prompt` já usado por `new_page_action`) e chama
      `api::create_page_with_type(vault, title, tipo)` — reaproveita o
      helper `prompt_title_and_create` se der pra generalizar (hoje
      só lida com `Option<String>` de template; ver se compensa
      estender ou se um caminho separado e mais simples é melhor,
      decidir na implementação)
- [ ] Digitar "kanban"/"calendário"/"grafo" etc na paleta filtra e
      acha o comando certo (filtro de texto já existe, só precisa que
      o label do comando contenha a palavra)
- [ ] `cd ui && cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam
- [ ] Validação ao vivo via MCP `tauri`: Ctrl+K, digitar "grafo",
      escolher "Nova página: Grafo de conexões", confirmar que a
      página nasce já com `type: graph` no frontmatter (sem precisar
      passar pelo painel de Propriedades)

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
