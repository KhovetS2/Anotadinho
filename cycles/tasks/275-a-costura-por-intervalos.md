---
id: "275"
titulo: "A costura por intervalos: fidelidade total"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["274"]
estima_min: 240
---

# 275 — A costura por intervalos

## O que o ciclo 274 deixou

35 de 242 páginas voltavam idênticas. O diagnóstico: guardar o TEXTO de
cada unidade não bastava porque a perda estava nas FRONTEIRAS — o que
existe entre as unidades não pertence a nenhuma e sumia.

## O desenho

Cada unidade de primeiro nível registra a FAIXA DE BYTES de onde veio, e
escrever vira costura: para cada unidade, ou os bytes originais ou a
serialização — e tudo que está ENTRE elas volta verbatim.

Isso dá fidelidade sem analisador perfeito: o que o analisador não
entendeu continua sendo copiado.

## Critérios de aceite

- [x] `embed::segment_com_intervalos` devolve os offsets que já existiam
      lá dentro, sem quebrar quem usa `segment`
- [x] Cada unidade registra `intervalo`, somado ao início do segmento
- [x] `escrever_costurando` costura, e o que está entre as unidades
      volta verbatim
- [x] Editar UMA unidade não reformata as vizinhas
- [x] 242/242 páginas do vault voltam idênticas

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored --nocapture
node scripts/uitest/run.mjs
```
