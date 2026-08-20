---
id: "174"
titulo: "Navegação por blocos dentro do editor"
status: pending
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: ["165"]
estima_min: 150
agente_alvo: claude-opus-5
---

# Navegação por blocos dentro do editor

## Objetivo

Pedido do usuário, vindo de testar o nav-mode: dentro do editor não há
destaque de região nenhum, e não dá pra andar pelo conteúdo com
Enter/setas — o item de topo `editor` é um *delegate*, ele joga o foco
no `contenteditable` e sai da frente (`app.rs` l.876). O ciclo 165
resolveu isso só pros embeds, com uma tecla dedicada (`Ctrl+.`), o que
é uma exceção e não um modelo.

O modelo que o usuário propôs, e que este ciclo implementa: **o
conteúdo da página é uma lista de blocos** — cada parágrafo, título,
lista, citação, bloco de código E cada embed é um item navegável. As
setas andam pelos blocos com destaque visual; Enter num bloco de TEXTO
põe o cursor dentro dele (modo de inserção); Enter num EMBED entra no
grupo de controles dele (o que o 165 já faz); Escape sobe um nível.

**Sem mudar o arquivo.** Os blocos são derivados na renderização (o
`.md` continua idêntico, sem id nem marcação nova) — ver Notas.

## Critérios de aceite

- [ ] O `editor` deixa de ser `data-nav-delegate` e vira
      `data-nav-group`: entrar nele coloca o foco no PRIMEIRO bloco, não
      dentro do texto
- [ ] Cada bloco do conteúdo é um item de navegação
      (`data-nav-item`/`data-nav-parent`), na ordem do documento:
      parágrafo, título, item de lista de topo, citação, bloco de
      código, divisor, imagem, tabela markdown e cada embed
- [ ] Destaque visual do bloco focado, reusando o indicador do ciclo
      139 (`nav-mode__item-active`) — é a falta que motivou o pedido
- [ ] Enter num bloco de TEXTO entra em inserção: cursor no fim do
      bloco, digitação normal, nav-mode suspenso
- [ ] Escape dentro de um bloco de texto volta pro nível de blocos (com
      o mesmo bloco destacado, não o primeiro)
- [ ] Enter num EMBED entra no grupo dele (comportamento do 165, agora
      alcançado pelo caminho comum em vez de tecla dedicada)
- [ ] `Ctrl+.`/`Ctrl+,` continuam funcionando como atalho de "pular pro
      próximo embed", agora como conveniência e não como único caminho
- [ ] Blocos derivados por POSIÇÃO, sem tocar no `.md`: reparsear o
      texto do segmento e mapear cada bloco pro nó de DOM
      correspondente
- [ ] Vim mode não conflita: com vim ligado, Escape no bloco de texto
      volta pro modo Normal do vim antes de sair pro nível de blocos
- [ ] Cheatsheet atualizado
- [ ] Validação ao vivo (MCP `tauri`): percorrer `painel.md` inteiro —
      título, callout, ações, 3 consultas, cronograma, colunas — só com
      setas/Enter/Escape, escrevendo num parágrafo no meio do caminho

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mudar COMO se digita (Enter criando bloco novo, Backspace fundindo
  blocos, arrastar bloco) — é a task 175, e é o pedaço arriscado
- Id de bloco no arquivo — task 176, e só sob demanda
- Numerar/decorar os blocos visualmente além do destaque de foco

## Notas

Por que dá pra fazer sem sujar o arquivo: o que este ciclo precisa é
saber "onde começa e termina cada bloco AGORA", e isso o parser já
sabe (`pulldown-cmark` devolve os eventos com offset). Id no arquivo
só é necessário pra REFERENCIAR um bloco específico ao longo do tempo
(transclusão, backlink por bloco) — e mesmo aí, escrito só no bloco que
alguém de fato referenciou (task 176).

A página já é uma lista de segmentos (`DocSegment`) — os embeds já são
blocos de primeira classe. O que falta é quebrar o segmento de markdown
(hoje um blocão contíguo por vez) em blocos menores para a navegação.
