---
id: "450"
titulo: "O instalador leva o anotadinho-cli, e a conversa avisa quando não há CLI"
status: done
criado: 2026-10-07
autor: agente
prioridade: media
depende_de: ["449"]
estima_min: 120
---

# 450 — CLI junto do app instalado

Spec: `pages/specs/cli-junto-do-app.md`. Proposta:
`pages/propostas/cli-junto-do-app.md`, abordagem **A**, escolhida pela
pessoa em 2026-10-07.

## Objetivo

O `anotadinho-cli` atende o MCP do agente, e o app o procura ao lado do
próprio executável. O instalador não o levava, então no app instalado o
agente rodava sem as ferramentas do vault, e ninguém percebia.

## Critérios de aceite

- [x] `tauri.bundle.json` com `externalBin`, aplicado só no
      empacotamento (`--config`). O `cargo build` do app não muda
- [x] `scripts/build.sh` copia o CLI com o sufixo do alvo e empacota com
      a config. No pacote cruzado do Windows, `anotadinho-cli.exe` vai
      junto do `anotadinho.exe`
- [x] `scripts/build.ps1` faz o mesmo no Windows
- [x] `localizar_cli` devolve `None` quando não há CLI ao lado nem no
      `PATH`, com o sistema por parâmetro (testes do ramo Windows)
- [x] Sem CLI, e com um agente que sabe receber MCP, o progresso mostra
      `AVISO_SEM_CLI` na janela e na TUI
- [x] O `.deb` gerado contém o CLI ao lado do app
- [ ] Conversa no app INSTALADO listando as ferramentas MCP: não feito.
      Instalar o `.deb` exige root, e quem sobe o app é a pessoa
- [ ] `build.ps1` e o `.msi`: não rodaram, porque não há Windows nem
      PowerShell aqui

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
(cd src-tauri && cargo tauri build --bundles deb --config tauri.bundle.json)
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
node scripts/uitest/run.mjs agente
```

## Não-objetivos

- Pôr o CLI no `PATH` do sistema
- Assinatura de código e atualização automática

## Notas

A proposta falava em `scripts/empacotar.(sh|ps1)`. O `build.sh` já era o
script de empacotamento e já copiava o CLI pra `target/release` (sem pôr
no instalador), então a mudança entrou nele, e o `.ps1` ganhou o nome
correspondente.
