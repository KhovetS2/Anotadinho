---
id: "268"
titulo: "Movimento dentro de qualquer embed"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["265", "267"]
estima_min: 180
---

# 268 — Movimento dentro de qualquer embed

## O buraco que o 265 abriu

O ciclo 265 calou o vim dentro do embed, e o 267 migrou só o calendário.
Nos outros nove, entrar virou beco sem saída: o vim se calava, o embed
não tratava, e só o Escape respondia. Um callout tem cinco controles e
nenhuma tecla os alcançava.

## A decisão: um padrão, não nove handlers

Em vez de escrever um handler por embed, o movimento dentro de um embed
ganhou um comportamento PADRÃO que todos herdam. Quem quer melhor
consome a tecla antes — é o que o calendário faz com os dias.

## Critérios de aceite

- [x] `j`/`k`/`h`/`l` andam entre os controles de qualquer embed
- [x] No kanban, `j` desce a COLUNA cartão a cartão
- [x] Item aninhado não é vizinho: descer de um cartão não cai no botão
      do próprio cartão
- [x] Escape sobe UM nível: do controle pra raiz do embed, da raiz pro
      bloco
- [x] O calendário segue consumindo antes (o `j` dele anda entre dias)
- [x] A escolha do vizinho mora no núcleo, com teste

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
