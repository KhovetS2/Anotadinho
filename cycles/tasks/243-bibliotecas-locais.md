---
id: "243"
titulo: "Mermaid e highlight.js vêm de casa, não da internet"
status: done
criado: 2026-08-30
autor: agente
prioridade: media
depende_de: []
estima_min: 30
---

# 243 — Bibliotecas locais

## Objetivo

`ui/index.html` buscava mermaid e highlight.js num CDN a cada abertura.
Num app de notas locais, isso fazia diagrama e realce de sintaxe
dependerem de conexão — e punha código de terceiros entrando na janela em
tempo de execução.

## Critérios de aceite

- [x] Os três arquivos moram em `ui/vendor/` e são copiados no build
- [x] Nenhum `script[src]` ou `link[href]` aponta para fora
- [x] Mermaid e highlight.js continuam funcionando

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs
```
