---
id: "335"
titulo: "Editar galeria, colunas, fluxo e consulta pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["334"]
---

# 335 — Editar galeria, colunas, fluxo e consulta

Pela gramática do vim (322):

- Galeria: `a`/`cc` legenda; `o`/`O` imagem nova pelo caminho (começa em
  `assets/`); `dd` tira; `yy`/`p` duplicam; `>>`/`<<` reordenam;
  `Ctrl+A`/`Ctrl+X` colunas da grade (1–6); `~` tamanho P → M → G.
- Colunas, no painel: `o`/`O` painel vazio ao lado (até 4); `dd` tira
  (fica um); `yy`/`p` duplicam; `>>`/`<<` reordenam; `Ctrl+A`/`Ctrl+X`
  largura (1–6fr); `i`/`a` começam a escrever no painel. Dentro, o
  markdown do 334.
- Fluxo: `Enter` numa transição move pra ela (a máquina de estados do
  núcleo decide); `>>` dá o avanço natural; `a`/`cc` editam a nota.
- Consulta: `a`/`cc` editam o recorte como linha de filtro
  (`from:pages status=done sort:-date group:type limit:10`, a mesma
  sintaxe do CLI — `Condition::parse` e `Query::linha_de_filtro` no
  núcleo); `~` gira a visão; `Ctrl+A`/`Ctrl+X` o limite.

No PRÓPRIO embed, `o`/`dd`/`yy`/`p`/`>>` continuam sendo do embed como bloco
da página (334).
