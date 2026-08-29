---
id: "230"
titulo: "Revisar vendo a página, não só o texto"
status: done
criado: 2026-08-29
autor: agente
prioridade: media
depende_de: ["204"]
estima_min: 90
---

# 230 — Revisar vendo a página

## Objetivo

A fila de revisão mostra só o diff em texto cru. Quem escreveu o embed lê
bem; quem só precisa decidir olha dez linhas de YAML e não sabe o que
está aprovando — e decidir é o que aquela tela existe para permitir.

## Critérios de aceite

- [x] Cada proposta alterna entre `Diff` e `Visualização`
- [x] Diff continua sendo o padrão ao abrir
- [x] A visualização renderiza o conteúdo PROPOSTO como a página fica,
      com markdown e embeds de verdade
- [x] Alternar não decide nada: a proposta segue pendente
- [x] O preview é inerte — não aceita interação, porque os botões de
      editar dos embeds não fariam nada ali

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Editar a proposta pela tela de revisão
- Diff lado a lado ou por palavra
