---
id: "261"
titulo: "A unidade de navegação e sua política"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["260"]
estima_min: 120
---

# 261 — A unidade e sua política

## O plano inteiro, e por que esta é a primeira

A spec `o-que-e-um-bloco` pede um modelo grande. Quebrado, com a ordem
escolhida por RISCO CRESCENTE — cada ciclo é verificável sozinho e a
suíte fica verde entre eles:

| ciclo | o que entrega | onde mexe | risco |
|---|---|---|---|
| **261** | a unidade, a política e o Iterator | só `crates/core` | nenhum: nada consome ainda |
| **262** | a cadeia de responsabilidade | só `crates/core` | nenhum: função pura |
| **263** | a lista única de blocos (RF1) | `ui/` navegação | médio |
| **264** | `dd`/`yy` em embed, com confirmação (RF2) | `ui/` vim | médio |
| **265** | vim DENTRO do embed pela cadeia (RF5) | os 10 `inline_*` | alto, mas incremental |
| **266** | Bridge do renderizador (a porta do CLI) | `ui/` + core | alto |

Os dois primeiros são de propósito invisíveis. É o que permite acertar o
vocabulário com teste puro antes de qualquer DOM entrar na conversa — e
foi exatamente o que faltou no ciclo 254, que implementou o vim direto
no DOM e pagou a semântica errada no 260.

## Escopo deste ciclo

`crates/core/src/unidade.rs`: a árvore (Composite), a política que cada
tipo declara, e a travessia (Iterator). Nada renderiza, nada navega.

## Sobre o `Block` que já existe

`crates/core/src/block.rs` NÃO é isto e não vai ser mexido. Ele é a
linha de markdown com `id::` e `depth`, usada por `Page` e pelo parser.
É lista plana, não conhece embed e não tem filhos.

Unificar os dois é desejável e fica como dívida declarada: exigiria
reescrever `parse_blocks` e `Page` junto, e misturar isso aqui faria o
ciclo tocar o parser de markdown no mesmo commit em que estreia o
modelo. A spec registra a dívida.

## Critérios de aceite

- [x] `Unidade` é uma árvore: tipo + filhos
- [x] Cada tipo declara `Politica` (aceita texto, cursor, filhos; é atômica)
- [x] Travessia em ordem de documento, com endereço (caminho) estável
- [x] A travessia NAVEGÁVEL não desce em unidade atômica — uma tabela é
      um destino, as células dela não são
- [x] `unidade_em(caminho)` devolve a unidade endereçada
- [x] Zero `web_sys`: o teste roda em `cargo test --workspace`

## Comandos de validação

```bash
cargo test --workspace
```
