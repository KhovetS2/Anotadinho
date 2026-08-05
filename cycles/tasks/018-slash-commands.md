---
id: "018"
titulo: "Comandos slash (/) para inserir blocos especiais"
status: done
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["017"]
estima_min: 50
agente_alvo: claude-sonnet
---

# Comandos Slash (/)

## Objetivo

Ao digitar `/` no início de uma linha, abre um menu flutuante com
opções de blocos: Título, Lista, Citação, Código, Tabela, Checklist,
Divisor (hr). Selecionar um item insere o bloco correspondente.

## Critérios de aceite

- [x] Menu slash aparece ao digitar `/` no início de linha
- [x] Opções: H1, H2, Lista, Checklist, Quote, Code, Table, HR
- [x] Navegação com setas + Enter seleciona
- [x] Escape fecha o menu
- [x] App continua compilando e abrindo
