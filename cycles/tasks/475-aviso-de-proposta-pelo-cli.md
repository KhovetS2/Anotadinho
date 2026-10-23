---
id: "475"
titulo: "O aviso de proposta pendente acende com proposta feita pelo CLI ou MCP"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
estima_min: 60
---

# 475 — Aviso de proposta pelo CLI

Relato da pessoa: as propostas da spec `tarefa-rica-home-e-memoria`, feitas
pelo CLI, não acenderam o ícone do topo da janela.

Causa: o watcher só marca mudança em `.md`, e a fila mora em
`.anotadinho/propostas/*.json`. O contador do cabeçalho só é recontado
quando o watcher acusa mudança. O cenário do 210 recarregava a janela
antes de olhar, e por isso não via.

## Critérios de aceite

- [x] Uma proposta criada ou resolvida em `.anotadinho/propostas/` acorda o
      app.
- [x] O `index.json` não acorda o app, porque isso seria um laço.
- [x] O `cli watch` continua emitindo só páginas.
- [x] Cenário de harness: `cli propor` com a janela aberta, SEM recarregar,
      e a contagem do aviso sobe; resolver, e ela volta.
- [x] O cenário foi visto reprovar no backend antigo.

## Comandos de validação

```bash
cargo test -p anotadinho-vault watcher
node scripts/uitest/run.mjs "(475)"
```
