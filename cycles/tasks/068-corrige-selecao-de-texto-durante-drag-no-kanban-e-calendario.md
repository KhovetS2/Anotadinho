---
id: "068"
titulo: "Corrige selecao de texto durante drag no kanban e calendario"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: ["067"]
estima_min: 30
agente_alvo: claude-sonnet
---

# Corrige seleção de texto durante drag no kanban e calendário

## Objetivo

Bug relatado pelo usuário: arrastando rápido (principalmente da direita
pra esquerda), o `mousedown` + mover o mouse selecionava o texto por
baixo do cursor, e o navegador tentava iniciar um drag nativo de conteúdo
por cima do nosso drag por mouse próprio — visualmente parecia "o ghost
de um container" em vez do card/evento sendo arrastado. Causa
identificada pelo próprio usuário.

## Critérios de aceite

- [x] `e.prevent_default()` nos handlers de `onmousedown` que iniciam o
      drag (kanban: card; calendário: barra de evento e bloco de horário)
- [x] `user-select: none` em `.kanban__card`, `.kanban__board`,
      `.calendar-grid__weeks` e `.calendar-grid__hour-scroll` (escopo
      restrito às áreas de drag — não no `.calendar-grid` inteiro, que
      também contém o `EventDetailModal`, que precisa continuar editável)
- [x] Validado ao vivo: arraste rápido de direita pra esquerda no kanban
      e no calendário, `window.getSelection().toString()` vazio durante
      todo o arraste, sem nenhum artefato visual estranho

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum — ciclo pequeno e focado só nesse bug

## Notas

Validação ao vivo simulou o arraste com uma sequência real de eventos
`mousemove` incrementais (8 passos, não só início+fim) pra reproduzir a
velocidade real do bug — confirmado `window.getSelection().toString()`
vazio do início ao fim do gesto, em ambos os componentes, e o card/evento
efetivamente moveu pra posição solta corretamente.
