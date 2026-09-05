---
id: "265"
titulo: "Vim dentro do embed, pela cadeia"
status: doing
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["262", "264"]
estima_min: 180
---

# 265 — Vim dentro do embed

## O modelo, em uma frase

Painéis aninhados: `j`/`k` andam ENTRE blocos; `Enter` ENTRA no bloco;
dentro dele as teclas são dele; `Escape` sai. É o tmux que a spec
descreve, e o nav mode já faz metade disso — falta valer com o vim
ligado.

## Como a cadeia se realiza aqui

A cadeia de responsabilidade do ciclo 262 não precisa de despacho
próprio: o DOM já borbulha. Um evento nasce no elemento focado e sobe
até o contêiner. Então:

- foco no WRAPPER do embed → o evento nasce fora do componente, sobe
  direto pro handler do documento, e o vim trata (é o `j`/`dd` dos
  ciclos 263 e 264);
- foco DENTRO do embed → o evento nasce no componente, ele trata o que
  quer e para a propagação; o que ele não quiser continua subindo e o
  documento trata.

`Interesses` (ciclo 262) é a forma declarativa dessa mesma regra, e vira
o teste puro dela.

## Critérios de aceite

- [ ] `Enter` num bloco atômico entra no embed
- [ ] Dentro do embed, o vim do documento NÃO age
- [ ] `Escape` volta pro wrapper, com o embed realçado
- [ ] O calendário anda entre dias com `j`/`k` quando está dentro
- [ ] Os outros nove embeds seguem sem mudança e com o comportamento de
      hoje — é a migração incremental prometida pelo desenho
- [ ] Cenários pra entrar, agir dentro, e sair

## Escopo

Um embed migrado (o calendário) como prova do mecanismo. Os outros
declaram nada e continuam como estão — não porque falta tempo, mas
porque é assim que a spec pediu que a migração acontecesse (RNF3).

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
