---
id: "512"
titulo: "TUI: selecionar no editor de código, indentar o trecho, recortar, copiar (OSC 52) e colar"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["506", "511"]
estima_min: 90
---

# 512 — Seleção no editor de código

O que o 506 deixou de fora: sem seleção, indentar um trecho era linha por
linha, e não dava pra mover código de lugar.

## Critérios de aceite

- [x] Shift + setas, Home, End, PgUp e PgDn selecionam, e andar sem Shift
      tira a seleção. Shift+↑/↓ chegam do terminal (`main.rs`).
- [x] Com seleção:
  - [x] Tab e Shift+Tab indentam e desindentam as linhas que ela toca;
  - [x] digitar ou Enter trocam o trecho;
  - [x] Backspace e Delete apagam.
- [x] Recortar, copiar e colar:
  - [x] Ctrl+X recorta e Alt+C copia; sem seleção, a linha inteira;
  - [x] Ctrl+V cola o que eles guardaram;
  - [x] a cópia vai também pra área de transferência do sistema, pelo
        OSC 52.
- [x] O trecho selecionado aparece com fundo.
- [x] Vale na aba Código do construtor.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
python3 copiar.py   # tmux com set-clipboard on: o buffer recebe o Alt+C
```

## Não-objetivos

- Ctrl+C pra copiar: no editor ele desiste da edição (390), e mudar isso
  agora pegaria quem já usa.
- Seleção pelo mouse: a TUI captura o mouse pra roda (454).
