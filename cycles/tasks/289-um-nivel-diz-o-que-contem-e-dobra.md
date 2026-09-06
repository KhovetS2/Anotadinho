---
id: "289"
titulo: "Um nível diz o que contém, e dobra"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["288"]
estima_min: 180
---

# 289 — Um nível diz o que contém, e dobra

Os dois primeiros itens da lista, e eles são o mesmo assunto: **a linha
de um nível é a forma fechada dele**.

## O que estava errado

Um `·` solto no meio da página — a lista, que é destino de navegação e
não tem texto próprio. E `[fluxo]` mudo, que é pior: o arquivo diz
`artefato: spec, etapa: concluida` e a tela não dizia nada.

O fluxo era o único embed sem filhos. Eu o deixei assim no ciclo 283
raciocinando que o estado dele já estava no texto — e no 284 parei de
mostrar o texto do embed, porque era o fence YAML inteiro. **Cada
decisão certa sozinha, erradas juntas.**

## O que a linha passa a dizer

- lista → `· 6 items`
- kanban → `[kanban] 2 columns`, com o nome que o próprio embed dá às
  partes dele (ciclo 283)
- fluxo → `[fluxo] Execução · Concluída`, porque partes de nomes
  DIFERENTES são campos, não coleção: contar "2 items" ali não diz nada

## E dobrar

A mesma linha, agora fechável. `z` ou espaço dobra o nível sob o
cursor; Enter num nível dobrado ABRE ele — pedir pra descer e não descer
seria o "fiquei preso" do ciclo 280 por outra porta.

**Nível com mais de 30 filhos nasce dobrado.** É a resposta ao caso que
a pessoa levantou: mil itens não se percorre de `j` em `j`. Dobrada, a
consulta ocupa uma linha, e a linha diz `1000 items`.

## Critérios de aceite

- [x] O fluxo declara artefato e etapa
- [x] Lista e embed dizem o que contêm
- [x] Partes de nomes distintos mostram os VALORES
- [x] Dobrar esconde o conteúdo e mantém a linha
- [x] Entrar num nível dobrado abre ele
- [x] Nível enorme nasce dobrado
- [x] A janela rola contando as linhas VISÍVEIS
- [x] A suíte da janela não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
