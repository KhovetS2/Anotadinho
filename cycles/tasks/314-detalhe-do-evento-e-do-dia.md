---
id: "314"
titulo: "O detalhe do evento e do dia no calendário"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["313"]
estima_min: 60
---

# 314 — O detalhe do evento e do dia

## O que motivou

Primeiro passo do "calendário completo" antes da edição: com o cursor
num evento não se via data, horário nem tags — o que a janela mostra no
modal do evento. E pra editar é preciso ver o que se vai mudar.

## O que mudou

O mecanismo de detalhe do 313 (parte `detalhe` escondida, mostrada no
pé da caixa com o cursor na parte) passa a valer no calendário:

- **Evento** (começo e continuação): `07/08/2026 · 14:30–15:15 · #infra`,
  ou `10/08/2026 → 14/08/2026 · 5 dias` numa barra de vários dias; o
  caminho da página quando o evento vem do vault.
- **Dia**: `sexta, 7 de agosto de 2026 · 1 evento`, contando também os
  que caíram no `+N mais`. O detalhe do dia não repete o número na frente.

O detalhe do dia é o ÚLTIMO filho, depois das faixas e do `+N mais`; o
desenho da semana o tira da conta das faixas.

## Critérios de aceite

- [x] Evento selecionado mostra datas, horário e tags
- [x] Dia selecionado mostra a data por extenso e quantos eventos
- [x] Faixas e navegação por dia/evento continuam iguais

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
