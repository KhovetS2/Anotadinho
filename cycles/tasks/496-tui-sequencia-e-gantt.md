---
id: "496"
titulo: "TUI: diagramas de sequência e gantt desenhados em caracteres"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["495"]
estima_min: 240
---

# 496 — Sequência e gantt no terminal

Terceira etapa da proposta `pages/propostas/tui-codigo-e-diagramas.md`
(RF6 pra sequência e gantt, RF7, RF8, RNF3).

## Critérios de aceite

- [x] `anotadinho_core::diagrama::texto::desenhar(&Diagrama, largura)`
      devolve linhas de trechos com papel, ou por que não desenha
      (`NaoCabe`, `SemDesenho`, `SemData`). Lê o mesmo modelo do construtor.
- [x] Sequência: caixas dos participantes (ator com cantos redondos), linha
      da vida, mensagens com rótulo e a seta de cada tipo, mensagem pra si
      em laço, notas à direita, à esquerda e sobre um ou dois.
- [x] Gantt: título, régua de datas, seções, barras no tempo (data,
      duração, `after`), textura própria pra `done`, papel por marca e
      marco como `◆`.
- [x] Largura em células: CJK conta dois. O que não cabe é recusado, não
      cortado.
- [x] Na TUI: dentro do modelo e cabendo, desenhado; fora do modelo, sem
      desenho ou largo demais, o código com uma linha de motivo.
- [x] Conferido no terminal de verdade (`scripts/tui/tela.py`).

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test -p anotadinho-tui
cargo test --workspace
cd ui && trunk build
python3 scripts/tui/tela.py pagina-com-diagramas.md --largura 110 --altura 50
```

## Não-objetivos

- Fluxograma e estados: é o 497 (até lá, código com o motivo).
