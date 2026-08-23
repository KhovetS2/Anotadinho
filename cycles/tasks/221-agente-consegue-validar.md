---
id: "221"
titulo: "O agente consegue rodar o harness"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["219", "220"]
estima_min: 45
agente_alvo: claude-opus-5
---

# O agente consegue rodar o harness

## Objetivo

O ciclo 220 chegou completo à validação e parou lá:
`Operation not permitted (os error 1)` ao abrir socket. Sem isso, todo
ciclo de UI feito por agente termina pela metade.

## Critérios de aceite

- [x] O preset do Codex libera rede, testado contra o binário
- [x] Configuração já gravada é migrada
- [x] O `AGENTS.md` diz pra NÃO tentar subir o app
- [x] O `AGENTS.md` diz o que fazer quando a ponte não responde

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

O sandbox `workspace-write` do Codex nasce sem rede, e o harness fala
com o app por WebSocket em `127.0.0.1:9223`. Testado com o binário: com
`-c sandbox_workspace_write.network_access=true`, o agente conecta.

O Codex não sabe liberar só o localhost — isto abre a rede inteira. É um
passo a mais sobre o `workspace-write`, que já deixa o agente editar o
código; quem não quiser, tira do preset.

O outro lado do problema era o agente TENTAR subir o app. `dev.sh` abre
uma janela e não retorna: num comando não-interativo trava, e num
sandbox nem começa. Quem deixa o app de pé é a pessoa; o agente só
conecta, e quando não consegue, pede e relata a validação como
pendente em vez de dar o ciclo por bom.
