---
id: "481"
titulo: "build.sh leva a TUI no pacote e ganha --instalar"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
estima_min: 40
---

# 481 — A TUI no build, e instalar com um comando

## Critérios de aceite

- [x] `scripts/build.sh` compila a TUI em release junto com o CLI.
- [x] O pacote (rpm/deb) instala `/usr/bin/anotadinho-tui`, pelo
      `externalBin` do `tauri.bundle.json`.
- [x] A TUI é linkada em `~/.local/bin` como o CLI, e vai pro
      `dist-windows/` quando o alvo do Windows está instalado.
- [x] `--instalar` gera só o pacote da distro e o reinstala (`dnf` ou
      `apt`). O resto dos argumentos vai pro `cargo tauri build`.
