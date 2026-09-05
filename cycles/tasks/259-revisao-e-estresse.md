---
id: "259"
titulo: "Revisão de padrões, complexidade e bateria de estresse"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["150", "171", "217", "251"]
estima_min: 300
---

# 259 — Revisão de padrões e estresse

## Objetivo

Revisar o projeto inteiro (45 mil linhas) procurando complexidade
errada, estrutura de dados errada e duplicação que pede um padrão — e,
principalmente, **medir** em vez de opinar. Nada aqui foi apontado por
leitura sozinha: cada achado tem um número antes e um número depois.

Junto, montar as duas baterias que faltavam: uma de backend
(`cargo test -p anotadinho-ipc --test estresse -- --ignored`) e uma de
UI (`node scripts/uitest/run.mjs --estresse`).

## Critérios de aceite

- [x] Bateria de estresse de backend, com vaults sintéticos de até 4 mil
      páginas, medindo varredura, consulta, agrupamento e cache
- [x] Bateria de estresse de UI, com páginas de 1200 blocos, tabela de
      600 linhas e parágrafo de 200 mil caracteres
- [x] A guarda de linearidade foi vista REPROVANDO a regressão que ela
      existe pra pegar, não só passando
- [x] Todo achado de complexidade tem medida antes/depois
- [x] A duplicação da ponte IPC vira um padrão, sem mudar assinatura de
      nenhuma função pública
- [x] `cargo test --workspace` e a suíte de 235 cenários continuam verdes

## O que NÃO foi feito, e por quê

- **Quebrar `editor.rs` (6080 linhas) em módulos.** É a maior dívida de
  organização do projeto, mas é mudança grande, mecânica e de risco alto
  num arquivo que concentra o histórico de bugs mais caro. Fica
  registrada no relatório com a divisão sugerida.
- **Tornar a extensão de seleção incremental.** Medido: 6,4ms por tecla
  numa página de 1200 blocos, dominado por ~1200 mutações de classe. A
  correção pediria guardar o alcance da seleção em estado Rust — e o
  módulo documenta ter rejeitado estado Rust de propósito, porque
  re-render o invalida. Medido e registrado, não mexido.

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-ipc --test estresse -- --ignored --nocapture
cd ui && cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --estresse
```
