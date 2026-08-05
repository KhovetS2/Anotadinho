---
id: "027"
titulo: "Undo/Redo (Ctrl+Z / Ctrl+Y) no editor"
status: pending
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["025"]
estima_min: 30
agente_alvo: claude-sonnet
---

# Undo/Redo

## Objetivo

Ctrl+Z desfaz, Ctrl+Y refaz. Interceptar as teclas e executar
`execCommand('undo')` / `execCommand('redo')` no contenteditable.

## Critérios de aceite

- [ ] Ctrl+Z desfaz última ação
- [ ] Ctrl+Y / Ctrl+Shift+Z refaz
- [ ] App continua compilando
