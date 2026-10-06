---
id: "449"
titulo: "O prompt vai pelo stdin: no Windows nenhuma conversa rodava"
status: done
criado: 2026-10-06
autor: agente
prioridade: alta
depende_de: ["448"]
estima_min: 180
---

# 449 — O prompt pelo stdin

Spec: `pages/specs/prompt-de-qualquer-tamanho.md`. Proposta:
`pages/propostas/prompt-pelo-stdin.md`. As duas estão na fila de
revisão. A execução começou a pedido direto da pessoa.

## Objetivo

Desde a 1.77.2, a std do Rust recusa `\r`/`\n` em argumento pra
`.cmd`/`.bat` (`make_bat_command_line`, lido no código-fonte). Todo
prompt do Anotadinho tem quebra de linha, e `claude`/`codex` do npm são
shims `.cmd`, então no Windows nenhuma conversa chegava a rodar. No
Linux, um argumento acima de 128 KiB dá `E2BIG`. O objetivo é entregar o
prompt pela entrada padrão.

## Critérios de aceite

- [x] `claude -p` e `codex exec -` leem o prompt de várias linhas do stdin
      (medido de verdade, inclusive `claude --resume` + `--mcp-config`)
- [x] Os perfis do Claude Code e do Codex vão pelo stdin, e o prompt não
      aparece nos argumentos
- [x] Um perfil gravado igual ao perfil pronto migra; um montado à mão não
- [x] Configuração antiga, sem o campo, continua pelo argumento
- [x] 300 KiB chegam inteiros a um processo de verdade (`cat`). Com a
      escrita sabotada (sem thread), o teste trava
- [x] O mesmo prompt como argumento é recusado no Linux (teste)
- [x] Erro `batch file arguments are invalid` vira uma frase que manda
      trocar pra stdin
- [x] "Entrega do prompt" na configuração da janela e da TUI; agente novo
      nasce pelo argumento
- [x] Harness: cenário novo, com 200 KiB e 2048 quebras pelo stdin
- [ ] Manual, numa máquina Windows: pendente
- [ ] opencode: continua pelo argumento, porque não foi medido se lê o
      stdin (ambiguidade apontada na spec)

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-vault processo
node scripts/uitest/run.mjs
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
```

## Não-objetivos

- `--input-format stream-json`
- CLI no instalador (spec `cli-junto-do-app`, que espera decisão)
