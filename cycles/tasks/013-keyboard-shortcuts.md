---
id: "013"
titulo: "Atalhos de teclado (Ctrl+N nova página)"
status: done
criado: 2026-08-05
autor: humano
prioridade: baixa
depende_de: ["012"]
estima_min: 15
agente_alvo: claude-sonnet
---

# Atalhos de teclado

## Objetivo

Ctrl+N cria nova página. Escape deseleciona página atual
(volta ao placeholder).

## Critérios de aceite

- [x] Ctrl+N abre prompt de nova página
- [x] Escape deseleciona página aberta
- [x] App continua compilando

## Comandos de validação

```bash
cd ui && trunk build
```
