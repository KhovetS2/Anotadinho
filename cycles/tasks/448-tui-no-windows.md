---
id: "448"
titulo: "TUI no Windows: preferências, ~, árvore de processos, fuso e caminho nativo"
status: done
criado: 2026-10-06
autor: agente
prioridade: alta
depende_de: ["255"]
estima_min: 120
---

# 448 — TUI no Windows

Spec: `pages/specs/tui-no-windows.md`. Proposta:
`pages/propostas/tui-no-windows.md`. As duas estão na fila de revisão. A
execução começou a pedido direto da pessoa, antes da aprovação.

Numeração: `cycles/tasks/` termina no 345, mas o histórico de commits vai
até o 447. A série 346–447 não tem arquivo de task, e este ciclo segue a
numeração dos commits.

## Objetivo

A TUI nasceu depois do ciclo 255 e não herdou as correções de
portabilidade dele. Este ciclo fecha os itens T1–T5 da spec.

## Critérios de aceite

- [x] Com `APPDATA` definido e sem `HOME`, as preferências ficam em
      `%APPDATA%\anotadinho\tui.json`
- [x] No Unix o caminho é exatamente o de antes (`XDG_CONFIG_HOME` ou
      `~/.config`)
- [x] `~` expande para `USERPROFILE` no Windows e `HOME` no Unix; um
      caminho sem `~`, ou com `~fulano`, não muda
- [x] A TUI, a janela e o `cli avaliar` encerram o agente pelo mesmo
      código (`anotadinho_vault::processo`)
- [x] No Linux, cancelar mata o processo neto. O teste foi visto
      reprovando quando o `killpg` foi tirado
- [x] "Hoje" não depende de `libc` nem de `#[cfg(unix)]` (`chrono::Local`)
- [x] O caminho entregue ao `explorer` só tem `\`
- [x] `cargo check --target x86_64-pc-windows-gnu` passa pra TUI, CLI,
      vault e app
- [ ] Manual, numa máquina Windows: pendente, porque o projeto não tem
      uma

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-vault processo
cargo check -p anotadinho-tui -p anotadinho-cli -p anotadinho-vault --target x86_64-pc-windows-gnu
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
node scripts/uitest/run.mjs agente
```

## Não-objetivos

- Entregar o prompt pelo stdin, que é o bloqueio real do agente no
  Windows. Fica pro ciclo 449.
- Pôr o CLI no instalador. A spec `cli-junto-do-app` espera decisão.
