---
title: Ciclo 167 — Operar kanban e cronograma pelo teclado
type: ciclo
ciclo: "167"
status: concluida
date: 2026-08-20
prioridade: media
depende_de: ["165"]
tags:
- ciclo
---

# Ciclo 167 — Operar kanban e cronograma pelo teclado

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Operar kanban e cronograma pelo teclado

## Objetivo

Navegar até o controle já será possível depois da task 165, mas duas
operações continuam exclusivas do mouse porque são ARRASTE: mover um
card entre colunas do kanban e mover/redimensionar uma barra do
cronograma. Enquanto isso existir, "usar o app só pelo teclado" é
meia-verdade.

## Critérios de aceite

- [x] Card de kanban focado: **Alt**+←/→ movem de coluna, Alt+↑/↓
      reordenam dentro da coluna. Alt (e não seta pura) porque as setas
      puras continuam navegando entre itens do nav-mode — mesma
      convenção de "mover linha" de editor de código (as duas coisas já existem em
      `KanbanEmbedData::move_card`, que resolve troca de coluna e
      reordenação numa chamada)
- [x] Barra de cronograma focada: Alt+←/→ deslocam por dia preservando
      a duração; Alt+Shift+←/→ esticam a ponta final
- [x] Item da gaveta "sem data": Enter agenda no início do período
      visível (já era o comportamento do clique, e o botão é focável)
- [x] Cada tecla mostra o resultado na hora e grava pelo mesmo caminho
      do arraste (`on_change`), então undo (095) desfaz igual
- [x] O que a tecla faz aparece no cheatsheet, na seção do embed
- [x] Validação ao vivo: montar um board e um cronograma e reorganizar
      os dois inteiros sem mouse

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar com o teclado no calendário (grade de horas) — entra depois
  se o padrão daqui funcionar
- Atalho pra criar card/etapa (já tem botão, alcançável por 165)

## Notas

`cargo test --workspace`: 260. Harness (177): 9/9, com cenário novo
conferindo que Alt+→ muda o card de coluna E que isso chega no disco.

Alt em vez de seta pura foi a decisão de desenho: seta pura continua
navegando (nav-mode dos ciclos 165/174), e Alt+seta move — é a mesma
convenção de "mover linha" do VS Code, e não rouba nenhuma tecla de
navegação.

A lógica de mutação toda já existe e é testada no core — este ciclo é
ligar tecla nela, não reimplementar movimento.

## Resultado

# Ciclo 167 - done

## Resumo

As duas operações que só existiam com o mouse — mover card entre
colunas do kanban e mover/esticar barra do cronograma — ganharam
teclado. Com isso o app é operável de ponta a ponta sem mouse.

Alt+setas movem; Alt+Shift+setas esticam a barra. Setas puras seguem
navegando (nav-mode).

## Arquivos criados/modificados

- `ui/src/components/embeds/inline_kanban.rs` — card vira item de
  navegação e trata Alt+setas
- `ui/src/components/embeds/inline_timeline.rs` — barra trata
  Alt+setas e Alt+Shift+setas
- `ui/src/components/cheatsheet_modal.rs` — as duas linhas novas
- `scripts/uitest/cenarios.mjs` — cenário novo

## Testes adicionados

- Cenário de harness: Alt+→ move o card pra coluna seguinte e a mudança
  chega no `.md`

## Problemas encontrados

- Nenhum: a mutação já existia e era testada no core (`move_card`,
  `move_item`, `resize_item`), então o ciclo foi só ligar tecla nela.

## Notas para próximos ciclos

- Restam 162, 163, 168, 169, 170, 176, 171, 172 (175 fora por decisão
  do usuário).
