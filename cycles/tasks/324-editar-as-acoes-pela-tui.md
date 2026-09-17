---
id: "324"
titulo: "Editar as ações pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["322"]
estima_min: 40
---

# 324 — Editar as ações pela TUI

Pela gramática do 322, num botão: `o`/`O` cria (ação `open-page`, como o
"+ ação" da janela), `i`/`a` renomeia, `dd` apaga, `yy`/`p` duplicam com
toda a configuração, `>>`/`<<` reordenam, `~` alterna o destaque.

## O que não entrou

Configurar a ação (template, página, campo, busca) — é o modal da janela.

## Critérios de aceite

- [x] Criar, renomear, apagar, colar, reordenar e destacar botões
- [x] Campos não editados (query, path…) sobrevivem
