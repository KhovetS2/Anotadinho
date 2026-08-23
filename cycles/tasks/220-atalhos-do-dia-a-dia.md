---
id: "220"
titulo: "Atalhos do dia a dia: aba fixa e criação de conversa"
status: done
criado: 2026-08-23
autor: agente
prioridade: media
depende_de: ["208"]
estima_min: 60
---

# Atalhos do dia a dia: aba fixa e criação de conversa

## Objetivo

Manter a página inicial como primeira aba, protegida contra fechamento e
visualmente identificada, e oferecer `conversa` junto aos demais tipos de
página na paleta de criação.

## Critérios de aceite

- [x] Com uma home definida, ela abre na primeira posição e não oferece fechar
- [x] Trocar a home reordena as abas sem perder o que estava aberto
- [x] `conversa` aparece na família de tipos do menu de criação e produz uma página utilizável
- [x] Cenário de harness cobre cada uma das duas frentes

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Fixar abas arbitrárias
- Reordenar abas por arraste

## Validação parcial

- `cargo test --workspace`: passou
- `PATH=/home/elis/.cargo/bin:$PATH NO_COLOR=true trunk build`: passou
- `cargo build --manifest-path src-tauri/Cargo.toml`: passou
- `cargo test --manifest-path ui/Cargo.toml tab_tests`: passou (2 testes)
- `node scripts/uitest/run.mjs`: **144/144** (rodado depois, com o app
  de pé — o agente não conseguiu por falta de rede no sandbox, ver nota)
- (registro original) não executado; o sandbox recusou abrir o
  servidor local do app com `Operation not permitted (os error 1)`

## Nota de fechamento

O ciclo foi implementado pelo agente (Codex) pelo fluxo do próprio
Anotadinho, e ele parou antes do status/commit por não conseguir rodar o
harness — decisão certa, e é o que o `AGENTS.md` manda fazer.

Validado aqui depois, com o app de pé: **144/144**, incluindo os dois
cenários permanentes que ele mesmo escreveu. Conferido também na tela:
a aba inicial é a primeira, tem `--fixed` e não oferece fechar; as
demais continuam com o ×; e "Nova página: Conversa" aparece na família
de tipos.

Ele também migrou os cenários correspondentes de `pendentes.mjs` pra
bateria permanente, que é o que aquele arquivo pede quando uma spec sai
do papel.

### Por que ele não conseguiu validar, e o que mudou

`Operation not permitted (os error 1)` ao abrir socket: o sandbox
`workspace-write` do Codex nasce sem rede, e o harness fala com o app
por WebSocket em `127.0.0.1:9223`.

Duas correções, no ciclo 221:

1. O preset do Codex passou a levar
   `-c sandbox_workspace_write.network_access=true`. Testado: com a
   flag, o agente conecta na ponte.
2. O `AGENTS.md` passou a dizer que o agente **não deve tentar subir o
   app** — `dev.sh` abre janela e não retorna — e que, sem a ponte, o
   caminho é pedir pra pessoa abrir e relatar a validação como
   pendente.
