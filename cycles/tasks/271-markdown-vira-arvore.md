---
id: "271"
titulo: "markdown → Unidade → markdown"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["261", "266"]
estima_min: 180
---

# 271 — markdown vira árvore, e volta

## Objetivo

Passos 1 e 2 da unificação: o corpo de uma página vira `Unidade`, e a
`Unidade` volta a ser markdown. Nada consome ainda — é o mesmo desenho
dos ciclos 261 e 262, acertar o vocabulário com teste puro antes de o
DOM entrar.

## Critérios de aceite

- [x] `analisar(corpo) -> Unidade`, com o embed como unidade atômica
- [x] `escrever(&Unidade) -> String`, reusando o `render::Markdown`
- [x] Ida e volta preserva a ÁRVORE (não o texto: markdown tem várias
      formas de escrever a mesma coisa)
- [x] A segunda volta é idêntica em TEXTO — senão salvar duas vezes daria
      diff em git pra sempre
- [x] O teste roda contra o VAULT REAL, não só contra fixtures meus

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored --nocapture
```
