---
id: "310"
titulo: "O evento de vários dias navegável no meio da barra"
status: done
criado: 2026-09-16
autor: agente
prioridade: media
depende_de: ["309"]
estima_min: 60
---

# 310 — Evento de vários dias navegável no meio da barra

## O que motivou

O 309 deixou anotado: o cursor só chegava num evento de vários dias
pelo dia em que ele começa na semana. No dia 12, no meio da sprint de 10
a 14, o dia só tinha a continuação da barra — e Enter não entrava.

## O que mudou

A continuação (`evento-continua`) passa a ser destino, como o começo:
`tela::e_evento` a inclui, e `cursor_passa_por_cima` só pula a faixa
vazia. No núcleo, a continuação leva o título do evento em vez de texto
vazio — ali, visto daquele dia, ela É o evento.

Com o cursor em qualquer dia da barra, a barra INTEIRA acende: o título
fica no dia em que ela começa na semana, e é ali que a cor do cursor
tem que aparecer.

## Critérios de aceite

- [x] Enter num dia no meio de uma barra pousa nela
- [x] `j`/`k` andam da continuação pros outros eventos do dia
- [x] A barra inteira acende, título incluído
- [x] A continuação carrega o título do evento

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
