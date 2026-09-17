---
id: "315"
titulo: "O calendário ancorado em hoje: um mês por vez, [ ] t e o dia de hoje marcado"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["314"]
estima_min: 180
---

# 315 — O calendário ancorado em hoje

## O que motivou

A TUI mostrava todo mês que tinha evento, empilhado. A janela mostra UM
mês — o de hoje, ao abrir —, com ‹ › pra trocar, "Hoje" pra voltar, o
dia de hoje marcado e a contagem de eventos no cabeçalho. Pra editar
(próximo passo) é preciso chegar em qualquer mês, inclusive os sem
evento, onde se vai criar um.

## O que mudou

**Núcleo.** A montagem do calendário virou `analise::partes_do_calendario
(dados, âncora, hoje)`. Sem âncora é o que `analisar` sempre fez (todo mês
com evento, o que o CLI mostra — o núcleo não lê relógio). Com âncora: um
mês só, com uma parte `cabecalho` na frente (a contagem), e o dia de hoje
como `dia-hoje`.

**TUI.** O `main` lê o dia de hoje no fuso local (`localtime_r`; UTC fora
do Unix) e o entrega ao `Estado` (`com_hoje`). Cada calendário da página
é remontado a partir da FONTE do embed, na sua âncora (`ancoras`, por
caminho; ausente é hoje). Trocar de página volta tudo pra hoje. Teste que
não chama `com_hoje` continua vendo o calendário sem âncora.

- `[` e `]` trocam de mês (os ‹ › da janela); `t` volta pro mês de hoje.
  Só com o cursor dentro de um calendário; o cursor vai pro mês.
- O cabeçalho mostra as teclas e a contagem à direita; o cursor passa por
  cima dele.
- O dia de hoje sai em pílula no tom de destaque — a janela pinta um
  círculo de `--accent-blue`; o cursor continua fundo cheio, pra os dois
  se distinguirem quando coincidem.
- `h`/`l` num evento sem vizinho no mês visível trazem o próximo (ou
  anterior) mês que tem evento e pousam nele — a navegação entre meses
  do ciclo 312, agora que só um mês está na tela.

## Diferença da janela

Abrir no mês de hoje é o que a janela faz, e num calendário cujos
eventos são de outro mês a grade aparece vazia até `[`/`]`. É de
propósito: o mesmo comportamento nos dois.

## Critérios de aceite

- [x] Com hoje conhecido, um mês por vez, o de hoje ao abrir
- [x] `[`/`]` trocam de mês, `t` volta pra hoje, só dentro do calendário
- [x] Dia de hoje marcado; cabeçalho com a contagem
- [x] `h`/`l` atravessam pro mês com evento seguinte/anterior
- [x] Sem hoje (CLI, testes antigos), o calendário de antes

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
