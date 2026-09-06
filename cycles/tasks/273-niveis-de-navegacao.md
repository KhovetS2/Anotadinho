---
id: "273"
titulo: "Níveis de navegação: andar não é entrar"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["268", "272"]
estima_min: 180
---

# 273 — Níveis de navegação

## O defeito relatado

Andando entre blocos com `j`, o cursor caía DENTRO de um calendário sem
ninguém ter pedido — e sair de lá exigia saber que existe um Escape.
Numa interface de terminal, sem mouse pra resgatar, isso prende quem
está usando.

A causa é minha, do ciclo 268: eu fiz o movimento DESCER quando não
havia vizinho na direção pedida. Parecia conveniente e é um beco.

## A regra

**Movimento nunca muda de nível.** Anda entre irmãos e PARA na borda.
Descer é um ato (`Enter`), subir é outro (`Escape`).

## Onde ela mora

Em `anotadinho_core::navegacao`, como pedido: a mesma regra pro editor,
pros dez embeds e pra uma futura interface de terminal. Um lugar só é o
que faz os três se comportarem igual sem combinarem nada.

## Critérios de aceite

- [x] `Cursor` + `Passo` + `mover` no núcleo, com o nível sendo o
      comprimento do caminho
- [x] Movimento para na borda; `None` é resposta, não falha
- [x] `Entrar` e `Sair` são os únicos que mudam de nível
- [x] `trilha()` diz onde a navegação está, pra GUI e pra CLI
- [x] Os itens de lista viram blocos, e a lista vira nível
- [x] O movimento do editor deixa de descer
- [x] A divergência conhecida do ciclo 272 FECHA, e o cenário que a
      afirmava vira o que afirma o acordo

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
