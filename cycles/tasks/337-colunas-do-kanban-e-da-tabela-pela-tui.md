---
id: "337"
titulo: "Colunas do kanban e da tabela pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["335"]
---

# 337 — Colunas do kanban e da tabela pela TUI

A coluna vira item como o painel de colunas: os comandos NELA são dela.

- Kanban, na coluna: `o`/`O` criam coluna ao lado (nome repetido é
  recusado), `a`/`cc` renomeiam (os cartões seguem), `dd` apaga com os
  cartões (fica pelo menos uma), `yy`/`p` duplicam com os cartões (a cópia
  ganha " (cópia)"), `>>`/`<<` mudam de lugar; colar um cartão copiado põe
  ele no fim da coluna. `Enter` numa coluna vazia cria o primeiro cartão —
  antes era o `o` na coluna. O pé mostra a tecla: "↵ + card" / "o + card",
  e "o + coluna".
- Tabela, no cabeçalho: `o`/`O` criam coluna de texto ao lado, `a`/`cc`
  renomeiam, `dd`/`x` apagam com as células (fica pelo menos uma), `yy`/`p`
  duplicam com as células, `>>`/`<<` mudam de lugar, `~` (e
  `Ctrl+A`/`Ctrl+X`) giram o tipo: texto, número, data, caixa, seleção,
  tags, url, página. Virar seleção/tags aproveita os valores que a coluna
  já tem como opções. Nas linhas, tudo como antes.
