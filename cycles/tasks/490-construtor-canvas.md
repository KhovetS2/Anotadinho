---
id: "490"
titulo: "Construtor mermaid: canvas de fluxograma e estados"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["489"]
estima_min: 360
---

# 490 — O canvas

Quarta e última etapa da proposta `pages/propostas/construtor-mermaid.md`.

## Critérios de aceite

- [x] Fluxograma e estados num canvas próprio, em SVG e sem biblioteca:
      arrastar um nó, puxar a ligação pela alça e selecionar nó ou
      ligação.
- [x] Painel do selecionado. **Nó:** texto e forma. **Ligação:** rótulo,
      tipo e inverter. Os dois se apagam por ali. Há também "+ Nó" e a
      direção.
- [x] Estados: início e fim fixos (do início só sai, no fim só chega), e
      início direto no fim não vira transição.
- [x] Pelo teclado (RF6): Tab chega no nó, as setas movem, `L` num nó e
      `L` noutro liga, Delete apaga e Escape cancela a ligação sem fechar
      o modal.
- [x] A posição arrastada vai no comentário `%% anotadinho-posicoes:`, e o
      canvas reabre com ela (RNF2).
- [x] Cenários de UI, por eventos de ponteiro sintéticos.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cd ui && cargo test --lib && trunk build
node scripts/uitest/run.mjs "(488)"
node scripts/uitest/run.mjs "(490)"
node scripts/uitest/run.mjs
```

## Não-objetivos

- Provar o arraste de verdade pelo harness. Ele fica para a conferência à
  mão (padrão "fronteira-do-sistema").
