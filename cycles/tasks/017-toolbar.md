---
id: "017"
titulo: "Toolbar de formatação Markdown"
status: done
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["016"]
estima_min: 40
agente_alvo: claude-sonnet
---

# Toolbar de formatação

## Objetivo

Adicionar toolbar entre o header e o editor com botões para inserir
formatação Markdown no texto selecionado.

## Critérios de aceite

- [x] Botões: Bold, Italic, Heading, List, Code, Quote, Link, Table
- [x] Inserem sintaxe Markdown no cursor/seleção
- [x] App continua compilando

## Comandos de validação

```bash
cd ui && trunk build
```
