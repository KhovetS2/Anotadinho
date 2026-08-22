---
title: Ciclo 114 — CLI empacotado junto no build da aplicacao
type: ciclo
ciclo: "114"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: ["110"]
tags:
- ciclo
---

# Ciclo 114 — CLI empacotado junto no build da aplicacao

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CLI empacotado junto no build da aplicação

## Objetivo

`scripts/build.sh` hoje só builda a GUI (`cargo tauri build`,
independente do crate `anotadinho-cli`, ciclo 110). Sem isso, quem
builda a aplicação pra instalar fica sem o binário do CLI — a peça
usada por um agente pra operar o vault sem GUI. Este ciclo faz
`scripts/build.sh` também buildar o CLI em release e deixar o binário
ao lado do bundle da GUI.

## Critérios de aceite

- [x] `scripts/build.sh` builda `anotadinho-cli` em release
      (`cargo build --release -p anotadinho-cli` a partir da raiz do
      workspace) além de rodar `cargo tauri build`
- [x] Binário do CLI copiado pra `src-tauri/target/release/` (mesmo
      diretório onde o binário da GUI e a pasta `bundle/` já ficam),
      pra quem pega a pasta de release levar os dois
- [x] Script continua funcionando do mesmo jeito se rodado de qualquer
      diretório (usa paths absolutos derivados de `BASH_SOURCE`, mesmo
      padrão já usado no script)
- [x] Mensagem de saída do script indica onde os dois binários ficaram

## Comandos de validação

```bash
./scripts/build.sh
ls -la src-tauri/target/release/anotadinho src-tauri/target/release/anotadinho-cli
```

## Não-objetivos

- Sidecar binary do Tauri (embutir o CLI DENTRO do bundle instalável,
  acessível de dentro do app) — fica pra depois se a GUI precisar
  invocar o CLI como subprocesso; por ora só precisa dos dois
  binários lado a lado na pasta de release
- Instalar o CLI no PATH do sistema automaticamente — fica a cargo do
  usuário copiar pra onde quiser

## Notas

`crates/cli` já é membro do workspace raiz (`Cargo.toml`), mas
`src-tauri` é excluído desse workspace (`exclude = ["ui", "src-tauri"]`)
e não depende do crate `anotadinho-cli` — são dois binários
independentes, sem overlap de build incremental além das deps
compartilhadas (`anotadinho-ipc` etc), que o cache do cargo já resolve
sozinho.

**Bug pré-existente encontrado e corrigido**: `cargo tauri build`
nunca tinha rodado até agora nesta sessão (toda validação anterior
usava `cargo build --manifest-path src-tauri/Cargo.toml`, que pula os
hooks do tauri-cli). `tauri.conf.json`'s `beforeBuildCommand` era
`"cd ../ui && trunk build --release"`, assumindo que o hook roda com
cwd = `src-tauri/` — mas provei empiricamente (dump de `env`/`pwd`
dentro do próprio hook) que o `tauri-cli` 2.11.4 executa os hooks com
cwd = raiz do repo, igual o `beforeDevCommand` (`"cd ui && ..."`).
Corrigido pra `"cd ui && ..."`, mesmo padrão do dev.

Também: `cargo tauri build` com `bundle.targets: "all"` tenta gerar
`.deb`/`.rpm`/`.AppImage`; o AppImage falha neste ambiente (sandbox
sem FUSE, `linuxdeploy` não roda) — `.deb`/`.rpm`/binário da GUI saem
certos. `scripts/build.sh` agora não aborta o script inteiro por causa
disso: verifica se o binário da GUI existe (aí sim é erro fatal) e só
avisa se algum formato de bundle específico falhou.

## Resultado

# Ciclo 114 - done

## Resumo

`scripts/build.sh` agora builda `anotadinho-cli` em release e copia o
binário pra `src-tauri/target/release/`, ao lado do binário e bundles
da GUI — sem isso, quem builda a aplicação pra instalar ficava sem a
peça usada por um agente pra operar o vault sem GUI.

## Arquivos criados/modificados

- `scripts/build.sh` — builda os dois binários, resiliente a falha de
  empacotamento de um formato específico
- `src-tauri/tauri.conf.json` — corrige `beforeBuildCommand` (bug
  pré-existente nunca exercido nesta sessão)

## Testes

`cargo test --workspace`: 101. `cd ui && cargo test --lib`: 75. Total 176.

Validação real: `./scripts/build.sh` rodado de ponta a ponta —
`anotadinho` (GUI) e `anotadinho-cli` presentes em
`src-tauri/target/release/`, CLI testado contra `VaultAnotadinho` de
verdade (`list-templates` retornou os 5 templates corretos).

## Notas

Dois bugs pré-existentes encontrados e corrigidos durante este ciclo,
ambos nunca exercidos antes (toda validação anterior usava `cargo
build --manifest-path src-tauri/Cargo.toml`, que pula os hooks do
tauri-cli):

1. `beforeBuildCommand` assumia cwd = `src-tauri/`, mas o tauri-cli
   2.11.4 roda os hooks com cwd = raiz do repo (mesmo que
   `beforeDevCommand`, que já estava certo) — corrigido.
2. `cargo tauri build` com `targets: "all"` tenta AppImage, que falha
   neste ambiente sandboxed (sem FUSE pro `linuxdeploy`) — script não
   aborta mais por causa disso, só avisa; `.deb`/`.rpm`/binário saem
   normalmente.

Próximo: filtros de busca no CLI (115) e escrita de propriedades (116),
implementados no mesmo lote de trabalho.
