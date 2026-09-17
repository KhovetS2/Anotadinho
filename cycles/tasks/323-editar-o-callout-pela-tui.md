---
id: "323"
titulo: "Editar o callout pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["322"]
estima_min: 60
---

# 323 — Editar o callout pela TUI

Pela gramática do 322:

- No título (ou no callout): `i`/`a`/`A` renomeia, `cc` do zero, `x`/`dd`
  apaga o título.
- Num bloco do corpo: `i`/`a` reescreve o markdown do bloco, `o`/`O` cria
  um bloco novo depois/antes, `dd` apaga, `yy`/`p` copiam. Num item de
  lista, o bloco novo é item vizinho e `cc` mantém a marca (`- `).
- Em qualquer lugar: `Ctrl+A`/`Ctrl+X` giram a variante, `~` alterna o
  "nasce recolhido".

Bloco de várias linhas não se reescreve pelo rodapé (é uma linha só): o
rodapé avisa.

## Critérios de aceite

- [x] Título, variante, recolhido
- [x] Criar, reescrever, apagar e colar blocos, inclusive itens de lista
- [x] O resto do arquivo não muda
