---
id: "200"
titulo: "Cenários das telas e fluxos sem cobertura"
status: done
criado: 2026-08-21
autor: humano
prioridade: media
depende_de: [198, 199]
estima_min: 120
agente_alvo: claude-opus
---

# Cenários das telas e fluxos sem cobertura

## Objetivo

Fechar as áreas que nenhum arquivo do harness tocava: journals,
páginas-de-TIPO (grafo, tags, assets, kanban — diferentes dos embeds de
mesmo nome), templates, exportação, git, histórico e cheatsheet.

Feito DEPOIS do 198 e do 199 de propósito: os cenários nascem já com
espera por condição, e não herdam as esperas lentas.

## Critérios de aceite

- [x] `scripts/uitest/telas.mjs` com 11 cenários.
- [x] Nenhum tempo fixo de setup.
- [x] Os cenários que CRIAM página no vault apagam o que criaram.
- [x] Suíte inteira verde, e `git status` do vault limpo ao fim.

## Comandos de validação

```bash
node scripts/uitest/run.mjs tela:
node scripts/uitest/run.mjs
```

## Não-objetivos

- Profundidade nessas telas: aqui é "abre e mostra o esperado". Quem
  precisar de detalhe ganha arquivo próprio depois.

## Notas

**"Nova página" é um fluxo de DOIS passos** — escolhe o template antes
de pedir o nome. O cenário original supunha um só e falhava esperando o
campo de título. Aproveitado pra cobrir templates, que era outra lacuna:
o teste confere que os templates do vault aparecem na lista.

**O título exibido é o SLUG**: digitar `__uitest_nova` abre
`uitest-nova`.

**Três comandos CRIAM página**: "Ir pra Hoje", "Ver Tags" e "Ver
Assets". Rodar a suíte sujava o vault com um journal e duas páginas de
índice. O helper `semSujarOVault` guarda o que existia antes e apaga só
o que o cenário criou.
