---
id: "301"
titulo: "O grafo por uma biblioteca consolidada"
status: done
criado: 2026-09-07
autor: agente
prioridade: alta
depende_de: ["300"]
estima_min: 120
---

# 301 — O grafo por uma biblioteca

## O questionamento estava certo

No ciclo 300 eu escrevi um Fruchterman-Reingold à mão. Perguntaram por
que não usar uma solução consolidada, e não havia boa resposta:

- o argumento de "evita dependência nova" vem do ciclo 120 e **não vale
  mais** — o projeto já vendoriza 3,2 MB de mermaid;
- o que eu escrevi é `O(n²)`; `d3-force-3d` usa Barnes-Hut, `O(n log n)`;
- e o que mais pesa não é o algoritmo: é o RENDER. Com 239 nós e giro
  contínuo, o SVG redesenhava 400 elementos por quadro.

## O que fazer

`3d-force-graph` vendorizado (1,3 MB, com three.js dentro). O
`graph_view.rs` vira montagem de dados + entrega. `crates/core/grafo.rs`
apagado — se precisar, o git guarda.

## O que muda nos testes, e é uma perda real

Com SVG eu afirmava coordenada: "os nós não estão todos no mesmo raio"
distinguia círculo de física. Com WebGL não há elemento por nó — o
desenho está numa textura.

O que sobra é o contrato entre app e biblioteca: os dados chegaram, o
canvas tem tamanho, clicar volta pro app. O layout é responsabilidade
dela, e testá-lo aqui seria testar a biblioteca.

## Critérios de aceite

- [x] A biblioteca é vendorizada, não buscada de CDN
- [x] As cores saem do CSS da janela
- [x] Biblioteca ausente escreve o motivo, não deixa tela branca
- [x] `grafo.rs` apagado
- [x] O canvas nasce com tamanho (3382×1824, WebGL)
- [x] A suíte não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
