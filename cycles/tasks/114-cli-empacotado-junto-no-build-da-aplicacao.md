---
id: "114"
titulo: "CLI empacotado junto no build da aplicacao"
status: done
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: ["110"]
estima_min: 30
agente_alvo: claude-sonnet
---

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
