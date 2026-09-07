---
id: "299"
titulo: "A sidebar em árvore de pastas"
status: done
criado: 2026-09-07
autor: agente
prioridade: alta
depende_de: ["298"]
estima_min: 120
---

# 299 — A sidebar em árvore

A TUI listava as páginas achatadas, e um vault com pastas virava uma
parede de nomes.

## O que fazer

A hierarquia real dos diretórios sob `pages/`, com pasta que abre e
fecha, na MESMA ordem da janela: pastas primeiro, alfabética, páginas
depois. Duas telas com a mesma ordem é uma coisa a menos pra pessoa
reaprender.

## Critérios de aceite

- [x] Pastas antes das páginas soltas, em ordem alfabética
- [x] Pasta fechada esconde o que tem dentro
- [x] O recuo conta a profundidade
- [x] `l`/`h` e Enter abrem e fecham a pasta
- [x] Enter numa página continua abrindo a página
- [x] Tudo nasce fechado, como na janela
- [x] Com busca, a hierarquia sai da frente

## Comandos de validação

```bash
cargo test -p anotadinho-tui
```
