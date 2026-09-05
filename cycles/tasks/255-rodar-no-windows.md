---
id: "255"
titulo: "Rodar no Windows: o que bloqueia, resolvido"
status: done
criado: 2026-09-05
autor: agente
prioridade: media
depende_de: ["237", "239", "241", "247"]
estima_min: 180
---

# 255 — Rodar no Windows

## Objetivo

Sair do diagnóstico e resolver o que ele levantou. O ciclo 237 mapeou
cinco bloqueios e dois pontos que degradam; os ciclos 239 e 241 já
fecharam dois deles pelo caminho, sem estarem mirando isso.

Estado ao começar:

| Item | Situação |
|---|---|
| B1 — shim `.cmd` do npm | aberto |
| B2 — caminho com espaço recusado | **fechado no ciclo 241** |
| B3 — sem campo pra editar o binário | **fechado no ciclo 239** |
| B4 — separador de caminho | aberto |
| B5 — `beforeDevCommand` POSIX | aberto |
| D1 — `kill` não alcança os netos | aberto |
| D2 — `strip_prefix` de caminho UNC | aberto |

## Critérios de aceite

- [x] B1: `claude` instalado como `claude.cmd` é encontrado e executado
- [x] B4: um caminho do sistema vira caminho do vault num lugar só, e a
      forma com `/` é a única que circula daí pra dentro
- [x] B5: `beforeDevCommand` e `beforeBuildCommand` que o `cmd.exe` roda
- [x] D1: cancelar ou estourar o tempo limite encerra a árvore, não só o
      filho direto — nos dois sistemas
- [x] D2: raiz canônica (`\\?\C:\...`) e evento do `notify` (`C:\...`)
      recortam igual
- [x] `contornar_travamento_nvidia` some fora do Linux
- [x] O app inteiro passa em `cargo check --target x86_64-pc-windows-gnu`
- [x] A suíte de harness continua verde no Linux

## O que NÃO se pode afirmar

Nada disto foi executado numa máquina Windows — o projeto não tem uma. O
que existe é checagem de tipos cruzada (que compila os ramos
`#[cfg(windows)]` de verdade, e por isso pega erro de API) e teste
unitário do que é lógica pura, com o sistema entrando por parâmetro em
vez de `cfg!`, justamente pra o comportamento do Windows ser exercido
aqui. Isso cobre "compila" e "a regra está certa". Não cobre "roda".

A spec vira `concluida` com esse limite escrito nela, não escondido.

## Comandos de validação

```bash
cargo test --workspace
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs
```
