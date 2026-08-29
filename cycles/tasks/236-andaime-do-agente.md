---
id: "236"
titulo: "O andaime que o modelo consome"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["229", "233"]
estima_min: 120
---

# 236 — O andaime que o modelo consome

## Objetivo

Melhorar a estrutura em volta do Anotadinho para o modelo trabalhar com
mais assertividade e escrever de forma mais concisa. Não é a suíte de
testes: é o que o modelo lê e usa antes de agir.

## Critérios de aceite

- [x] `anotadinho-cli contexto` devolve o mapa do vault numa chamada:
      pastas, tipos, fluxo por etapa, fila de revisão, padrões e prompts
- [x] `--json` para consumo programático
- [x] `AGENTS.md` abre com "os primeiros noventa segundos", e a ordem é
      contexto → spec → padrões → código
- [x] `AGENTS.md` diz como fechar o trabalho (`cli etapa`)
- [x] `AGENTS.md` tem uma seção sobre como responder
- [x] Prompts padrão cobrem planejar e entender código, além de
      investigar bug, revisar spec e escrever cenário
- [x] Os prompts novos entram na semente do vault

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
