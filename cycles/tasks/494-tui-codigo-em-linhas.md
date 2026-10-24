---
id: "494"
titulo: "TUI: bloco de código em várias linhas"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: []
estima_min: 120
---

# 494 — O bloco de código em linhas

Primeira etapa da proposta `pages/propostas/tui-codigo-e-diagramas.md`
(spec `pages/specs/tui-codigo-e-diagramas.md`, RF1, RF3, RF4, RF5, RF9).

## Critérios de aceite

- [x] O bloco aparece numa caixa com a linguagem no topo ("código" sem
      linguagem), com as linhas e a indentação; tabulação vira 4 espaços.
- [x] Mais alto que 30 linhas (ou que o painel), corta com
      "… mais N linha(s) — Enter abre".
- [x] Linha mais larga que o painel é cortada com `…`, sem quebrar.
- [x] O bloco é uma parada só do cursor, e Enter continua abrindo o modal.
- [x] O bloco de código dobra: uma linha com a linguagem e o tamanho.
- [x] Conferido num terminal de verdade (`scripts/tui/tela.py`, tmux).

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo test --workspace
python3 scripts/tui/tela.py pagina-com-codigo.md
```

## Não-objetivos

- Cor por linguagem: é o 495.
