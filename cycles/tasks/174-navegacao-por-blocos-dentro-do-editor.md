---
id: "174"
titulo: "Navegação por blocos dentro do editor"
status: done
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

- [x] O `editor` deixa de ser `data-nav-delegate` e vira
      `data-nav-group`: entrar nele coloca o foco no PRIMEIRO bloco, não
      dentro do texto
- [x] Cada bloco do conteúdo é um item de navegação
      (`data-nav-item`/`data-nav-parent`), na ordem do documento:
      parágrafo, título, item de lista de topo, citação, bloco de
      código, divisor, imagem, tabela markdown e cada embed
- [x] Destaque visual do bloco focado, reusando o indicador do ciclo
      139 (`nav-mode__item-active`) — é a falta que motivou o pedido
- [x] Enter num bloco de TEXTO entra em inserção: cursor no fim do
      bloco, digitação normal, nav-mode suspenso
- [x] Escape dentro de um bloco de texto volta pro nível de blocos (com
      o mesmo bloco destacado, não o primeiro)
- [x] Enter num EMBED entra no grupo dele (comportamento do 165, agora
      alcançado pelo caminho comum em vez de tecla dedicada)
- [x] `Ctrl+.`/`Ctrl+,` continuam funcionando como atalho de "pular pro
      próximo embed", agora como conveniência e não como único caminho
- [x] Blocos derivados por POSIÇÃO, sem tocar no `.md`: reparsear o
      texto do segmento e mapear cada bloco pro nó de DOM
      correspondente
- [x] Vim mode não conflita: o Escape do vim (sair do modo Inserção) é
      tratado ANTES e já parava a propagação, então a subida pro nível
      de blocos só acontece quando o vim não quis a tecla
- [x] Cheatsheet atualizado
- [x] Validação ao vivo (MCP `tauri`): percorrer `painel.md` inteiro —
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

`cargo test --workspace`: 260. `cd ui && cargo test --lib`: 26.
Harness (ciclo 177): 8/8, incluindo o cenário novo de blocos.

Mudança de comportamento herdada pelo 165: Escape dentro de um embed
agora volta pro NÍVEL DOS BLOCOS com o embed destacado, em vez de
cair direto no texto — quem entrou pelo teclado continua no teclado,
sem perder o lugar. O cenário do harness foi atualizado junto.

**O harness pagou o custo dele no primeiro uso.** O cenário da recarga
automática (173) começou a falhar e expôs um bug pré-existente no
`app.rs`: o `Interval` do watcher capturou o handle de `list_version`
na criação, então `*list_version + 1` devolvia sempre o mesmo número —
o watcher avisava UMA vez por sessão e depois nunca mais. Corrigido com
um contador fora do handle. É o mesmo modo de falha do `edited_ref` do
editor e do arraste do ciclo 155.

Uma armadilha do próprio harness também virou comentário: escrever o
arquivo de fixture faz o watcher acusar mudança, e a recarga do 173
chegava no meio do cenário desfazendo o que ele tinha feito. O
`recarregar()` agora espera essa poeira baixar.

Por que dá pra fazer sem sujar o arquivo: o que este ciclo precisa é
saber "onde começa e termina cada bloco AGORA", e isso o parser já
sabe (`pulldown-cmark` devolve os eventos com offset). Id no arquivo
só é necessário pra REFERENCIAR um bloco específico ao longo do tempo
(transclusão, backlink por bloco) — e mesmo aí, escrito só no bloco que
alguém de fato referenciou (task 176).

A página já é uma lista de segmentos (`DocSegment`) — os embeds já são
blocos de primeira classe. O que falta é quebrar o segmento de markdown
(hoje um blocão contíguo por vez) em blocos menores para a navegação.
