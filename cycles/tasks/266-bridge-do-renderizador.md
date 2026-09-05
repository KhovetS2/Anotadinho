---
id: "266"
titulo: "Bridge do renderizador: a porta do CLI"
status: done
criado: 2026-09-05
autor: agente
prioridade: media
depende_de: ["261"]
estima_min: 120
---

# 266 — Bridge do renderizador

## Objetivo

A costura entre o que uma página É (a árvore de `Unidade`) e como ela é
DESENHADA. É o que torna a versão CLI possível — e, antes disso, o que
permite testar estrutura sem navegador.

## Critérios de aceite

- [x] `Renderizador` como trait, com travessia que não sabe desenhar
- [x] Duas implementações MUITO diferentes: markdown e árvore de terminal
- [x] Cada renderizador decide se desce em unidade atômica
- [x] `sair` chamado depois dos filhos, pra abrir/fechar delimitador
- [x] Zero `web_sys`

## O que este ciclo NÃO faz

Não porta o editor Yew pra este trait. O editor é a maior implementação
do projeto e migrá-lo é trabalho de outra ordem. O que entra aqui é a
COSTURA e a prova de que ela aguenta duas saídas sem relação uma com a
outra.

Dizer isso importa: um trait sem segunda implementação é decoração, e
teria passado despercebido como "arquitetura pronta".

## Comandos de validação

```bash
cargo test --workspace
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
```
