---
id: "291"
titulo: "A gramática do vim ligada na TUI"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["289"]
estima_min: 90
---

# 291 — A gramática do vim ligada

## O que este ciclo NÃO escreveu

A gramática. Ela mora em `anotadinho_core::vim` desde o ciclo 285 —
contagem, operador, movimento, `gg`/`G`, o alcance inclusivo/exclusivo
tirado do Neovim no 260 — com 19 testes, e **ninguém a consultava**.

A TUI lia tecla a tecla num `match` e ignorava tudo isso.

## O que ele escreveu

A tradução: comando fechado → passos de navegação. `Mover(Baixo, 10)`
vira dez `Passo::Proximo`, e a régua de "dá ou não dá" continua sendo da
árvore (ciclo 281).

## Decisões

**Só movimento.** `dd`, `yy`, `ciw` fecham comando e não fazem nada: a
TUI é de leitura. O importante é CONSUMIR — deixar vazar faria o segundo
`d` cair no tratamento de tecla e disparar outra coisa.

**Seta continua valendo.** A gramática devolve `Ignorada` pra elas, e o
caminho de sempre trata. Quebrar a seta pra ganhar `10j` seria troca
ruim.

**Parar na borda.** `1000j` numa página de cinco blocos para no último
em vez de custar mil travessias da árvore.

**Mostrar o que está pela metade.** Teclar `1` `0` e não ver nada faz a
pessoa achar que a tecla não pegou. O rodapé mostra `10`, como o canto
do vim.

## Critérios de aceite

- [x] `10j`, `3k` andam a contagem
- [x] `gg` e `G` vão pras pontas; `10G` é a décima linha
- [x] A contagem fica pendente até o movimento chegar
- [x] Seta e `z` continuam funcionando
- [x] Operador é consumido sem fazer nada
- [x] O pendente aparece na tela

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo clippy -p anotadinho-tui --all-targets
```
