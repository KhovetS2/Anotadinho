---
id: "327"
titulo: "Testes de tela da TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
estima_min: 60
---

# 327 — Testes de tela da TUI

`crates/tui/tests/telas.rs`: cada CENA é uma página (os embeds de
`VaultAnotadinho/pages/exemplos-embeds.md`, copiada em
`tests/telas/paginas/embeds.md`), uma sequência de teclas e um tamanho.
A foto `tests/telas/<cena>.tela` guarda o texto da tela, uma grade de
letras com o estilo de cada célula (legenda embaixo) e o arquivo gravado
quando as teclas editam. Foto diferente vira `<cena>.tela.nova`.

```bash
cargo test -p anotadinho-tui --test telas
ATUALIZAR_TELAS=1 cargo test -p anotadinho-tui --test telas
TELAS_SO=kanban TELAS_ANSI=/tmp/ansi cargo test -p anotadinho-tui --test telas
```

As fotos só são regravadas depois de conferir o desenho contra a janela
(prints tirados do DOM vivo do app no dev server).
