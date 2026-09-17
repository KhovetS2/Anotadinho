---
id: "316"
titulo: "As visões Semana e Dia do calendário"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["315"]
estima_min: 180
---

# 316 — As visões Semana e Dia

## O que motivou

A janela tem o seletor Mês/Semana/Dia; a TUI só tinha o mês.

## O que mudou

**Núcleo.** `analise::Visao` e `partes_do_calendario_na_visao(dados,
visão, âncora, hoje)`:

- **Semana**: a semana (domingo a sábado) da âncora, como um "mes" de
  uma semana só (`9 – 15 de agosto de 2026`; atravessando o mês, `30 de
  agosto – 5 de setembro de 2026`). Todas as faixas, sem `+N mais`
  (`calendario::pack_days_ate`, com o limite como parâmetro — a janela
  continua com 3), o horário na frente do título, e dentro do dia os de
  dia inteiro primeiro e os com hora pela hora.
- **Dia**: uma "agenda" com um "compromisso" por evento que toca o dia —
  dia inteiro primeiro, depois pela hora —, a hora numa parte "hora" e o
  detalhe do 314. Dia vazio: "sem eventos", que não é destino.

Todo dia da grade passou a carregar a própria data numa parte "data":
numa semana solta, a posição do dia não diz mais a data. `tela` lê as
datas dali (antes lia pelo rótulo do mês e pela posição).

**TUI.** `m` troca a visão (Mês → Semana → Dia) na data do cursor — num
dia ou evento, a semana e a agenda são as dele. `[`/`]` andam um mês,
uma semana ou um dia, conforme a visão. O cabeçalho mostra a visão. Na
agenda, `j`/`k` andam entre compromissos e `h`/`l` vão pro dia anterior
ou seguinte que tem evento; na semana, `h`/`l` saem da semana pra a
próxima com evento.

## Critérios de aceite

- [x] `m` alterna Mês/Semana/Dia mantendo o dia do cursor
- [x] `[`/`]` andam na unidade da visão
- [x] Semana sem limite de faixas, com horário e ordem pela hora
- [x] Agenda do dia ordenada, com hora, detalhe e "sem eventos"
- [x] `h`/`l` na agenda e na semana atravessam pra onde há evento

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
cargo check --target wasm32-unknown-unknown --manifest-path ui/Cargo.toml
```
