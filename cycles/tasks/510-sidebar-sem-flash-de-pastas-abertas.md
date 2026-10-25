---
id: "510"
titulo: "Sidebar: ao abrir o app, as pastas não aparecem abertas por um instante"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: []
estima_min: 45
---

# 510 — O flash das pastas abertas

O cenário "(200) página type: grafo" reprovou de vez em quando, com duas
assinaturas: "esperava a página 'grafo' na sidebar" e "esperava a pasta
/produto abrir".

## Causa (medida)

Sondei a sidebar depois de um reload, de 100 em 100 ms, abrindo a
`/produto` como o cenário faz. Numa das seis voltas, a pasta estava
ABERTA aos 240 ms e FECHADA aos 347 ms, sem clique nenhum.

O efeito de carga punha a lista das páginas (`pages.set`) antes de
esperar a das pastas. O quadro do meio tinha as páginas e nenhuma pasta
recolhida, então tudo aparecia aberto, e o recolhimento vinha logo
depois.

O comentário do ciclo 207 dizia justamente que o recolhimento era feito
"no mesmo lote" pra evitar esse flash. O `await` no meio quebrava isso.

- O harness que via a pasta aberta não clicava, e ela fechava sozinha.
- O que clicava no quadro aberto desfazia o recolhimento, e a pasta não
  abria.
- A pessoa via um piscar de tudo aberto ao abrir o app.

## Critérios de aceite

- [x] As duas listas chegam antes de qualquer `set`.
- [x] Cenário (510): 5 reloads. Em cada um, observar a sidebar a partir do
      primeiro item e nenhuma pasta pode abrir e fechar sozinha.
      Reprovou com a ordem antiga ("pages/ciclos, pages/conversas… abriram
      e fecharam sozinhas"), passou com a nova.
- [x] Suíte inteira verde (295/296; o 180 não se aplica ao gerenciador em mosaico).

## Comandos de validação

```bash
node scripts/uitest/run.mjs "(510)"
node scripts/uitest/run.mjs "(200)"
node scripts/uitest/run.mjs
```
