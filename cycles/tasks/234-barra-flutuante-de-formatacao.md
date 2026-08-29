---
id: "234"
titulo: "Barra flutuante de formatação"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: []
estima_min: 150
---

# 234 — Barra flutuante de formatação

## Objetivo

O editor não tinha formatação por interface. Negrito e itálico só saíam
digitando `**` e `*`, e quem não sabe markdown não tinha como descobrir
que existiam. O menu `/` só oferece blocos.

## Critérios de aceite

- [x] Selecionar texto no editor abre uma barra sobre a seleção
- [x] Selecionar fora do editor não abre nada
- [x] Esvaziar a seleção fecha a barra
- [x] Negrito, itálico, tachado, código e link
- [x] Clicar de novo na mesma marca tira a marca
- [x] A marca sobrevive ao salvar — chega ao arquivo como markdown
- [x] Sem `execCommand`

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Cor e realce (é o ciclo 235)
- Atalhos de teclado (`Ctrl+B`)
