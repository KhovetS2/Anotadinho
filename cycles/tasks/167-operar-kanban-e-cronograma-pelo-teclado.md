---
id: "167"
titulo: "Operar kanban e cronograma pelo teclado"
status: done
criado: 2026-08-20
autor: humano
prioridade: media
depende_de: ["165"]
estima_min: 90
agente_alvo: claude-opus-5
---

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
