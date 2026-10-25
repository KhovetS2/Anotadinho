---
id: "511"
titulo: "TUI: colar do terminal entra inteiro (bracketed paste), sem o Enter colado agir"
status: done
criado: 2026-10-24
autor: agente
prioridade: alta
depende_de: ["506"]
estima_min: 60
---

# 511 — Colar do terminal

Achado ao planejar a seleção no editor: a TUI não pedia ao terminal a
colagem em bloco. O texto colado chegava como uma tecla por caractere, e
o Enter colado AGIA:

- **no editor de código (506):** o recuo automático somava ao recuo do
  próprio texto, e cada linha colada entrava mais pra dentro;
- **numa conversa:** a mensagem era enviada na primeira quebra;
- **fora de um campo:** cada letra era um comando do vim.

## Critérios de aceite

- [x] `EnableBracketedPaste` ao entrar e `DisableBracketedPaste` ao sair;
      `Event::Paste` vai pra `app::colar`.
- [x] Editor de código e aba Código do construtor: o texto entra como
      veio (CRLF vira LF), num passo só do desfazer.
- [x] Inserção de bloco: entra no cursor, com a quebra virando espaço.
- [x] Conversa escrevendo: entra no rascunho, com as quebras, sem enviar.
- [x] Formulário, barra e filtro: uma tecla por caractere, sem o Enter.
- [x] Fora de um campo: não vira comando, e um aviso diz onde colar.
- [x] Medido na fronteira (tmux `paste-buffer -p` contra sem `-p`).

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
python3 colar.py p        # tmux paste-buffer -p
python3 colar.py teclas   # o caminho antigo, pra comparar
```

## Não-objetivos

- Reindentar o que se cola pelo recuo do cursor: o texto entra como veio.
- Copiar pra área de transferência do sistema.
