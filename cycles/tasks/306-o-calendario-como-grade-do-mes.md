---
id: "306"
titulo: "O calendário como grade do mês de verdade, com o algoritmo de faixas num lugar só"
status: done
criado: 2026-09-16
autor: agente
prioridade: alta
depende_de: ["305"]
estima_min: 240
---

# 306 — O calendário como grade do mês

## O que motivou

O 305 deixou o calendário como lista de selos, e anotou por quê: um mês
em 7 colunas não cabia no modelo, em que cada unidade vira LINHAS de
terminal. O pedido agora é a grade de verdade, igual à da janela.

## O algoritmo de faixas era da janela, e só dela

`pack_days` (em que faixa vai cada evento, e quanto transborda), a
montagem das 42 células do mês e os rótulos `D S T Q Q S S` moravam em
`ui/src/components/embeds/inline_calendar.rs`. Copiar pro terminal
criaria duas respostas pra "em que faixa vai este evento". Foram pro
núcleo (`crates/core/src/calendario.rs`), com o mesmo código, e a
janela passou a importar de lá — o mesmo movimento do ciclo 149 com
`embed` e `date_util`.

De quebra, o print do 305 estava errado num detalhe: o fixture usava
`DOM SEG TER…`, e a janela de verdade usa `D S T Q Q S S`. Achado
lendo o código pra mover.

## A árvore

`partes_do_embed` monta um `mes` por mês que tem evento, uma `semana`
(fileira) por linha da grade, e 7 `dia`/`dia-fora`. Dentro de cada dia,
uma parte por FAIXA da semana: `evento`/`evento--info`… (começa aqui,
com a cor de `badge_class` sobre as tags do calendário, igual à
janela), `evento-continua` ou `vazio`; e `mais` quando transborda.
Evento sem data vai pra `sem-data`, a gaveta.

As faixas são dado de desenho: `tela::fica_fora_da_tela` (antes
`e_geometria`) as tira da lista ANTES de achatar as fileiras — senão
uma faixa no meio da semana interromperia a fileira e os dias seguintes
deixariam de ser segmento. Enter num dia não desce nelas.

## O desenho

`linhas_do_mes` desenha nome, dias da semana e a borda de cima;
`linhas_da_semana` desenha números à direita da célula, uma linha por
faixa, `+N mais` quando sobra, e a régua (`├┼┤` ou `└┴┘`). Evento de
vários dias é UMA barra que atravessa as bordas dos dias, como
`grid-column` na janela. A cor é a pílula de `badge--*`: texto no tom e
fundo com 15% dele (`Tema::pilula`, o `color-mix` da janela feito à
mão, contra o fundo da tela).

## Diferenças da janela, de propósito

- Sem "mês corrente": o terminal não navega de mês, então mostra todo
  mês que tem evento.
- A sexta semana só entra quando tem dia do mês (a janela sempre
  reserva as seis).
- Sem destaque de "hoje" nem contagem de eventos no cabeçalho.

## Critérios de aceite

- [x] `pack_days`, `month_cells` e os rótulos moram no núcleo; a janela
      importa de lá e compila
- [x] O calendário vira grade: bordas dos dias alinhadas em todas as
      semanas, números à direita, dias de fora apagados
- [x] Evento de vários dias é uma barra só
- [x] Cor do evento = `badge_class` sobre as tags, tingida como na janela
- [x] Quarto evento do dia vira `+1 mais`
- [x] Evento sem data vai pra gaveta
- [x] Enter num dia não leva o cursor pra uma faixa sem linha
- [x] A grade não quebra num terminal estreito

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
cd ui && trunk build
```
