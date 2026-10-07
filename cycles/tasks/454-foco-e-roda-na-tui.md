---
id: "454"
titulo: "TUI: abrir pela barra leva o foco junto, e a roda do mouse rola"
status: done
criado: 2026-10-07
autor: agente
prioridade: alta
depende_de: ["453"]
estima_min: 120
---

# 454 — Foco e roda do mouse na TUI

Pedido direto da pessoa: "corrigir o foco da TUI" e "não dá para
scrollar no terminal".

## Objetivo

- Abrir algo pela barra de comandos, por uma escolha ou por um link
  deixava o foco onde estava. Nas propostas abertas por `:`, `v` e
  espaço não faziam nada até um `Tab`.
- A TUI não capturava o mouse. A roda ia pro terminal, que na tela
  alternativa não rola nada.

## Critérios de aceite

- [x] Todo `AbrirPagina`/`AbrirEspecial` atendido põe o foco no
      conteúdo, igual ao `Enter` da barra lateral
- [x] Mouse capturado ao entrar e liberado ao sair
- [x] A roda age onde o ponteiro está:
  - conteúdo: um bloco por passo, e a tela segue o cursor;
  - barra lateral: anda a lista;
  - modal de lista ou de leitura: age como as setas;
  - tela especial: rola o item e passa pro vizinho no fim dele
- [x] A roda não faz nada onde se digita (edição, busca, formulário,
      conflito)
- [x] O teste da proposta longa reprova com a rolagem sabotada
- [x] Pty: `:` → propostas → `v` funciona sem `Tab`; 40 passos de roda
      levam a tela de "1 a 26" pra "15 a 41"

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
```

## Não-objetivos

- Clique e arrasto (a TUI segue de teclado)
- Copiar pro sistema (a TUI não tem área de transferência)
