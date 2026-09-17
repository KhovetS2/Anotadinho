---
id: "313"
titulo: "O cronograma navegável: eixo de datas, h/l no tempo e o detalhe da barra"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["312"]
estima_min: 120
---

# 313 — O cronograma navegável

## O que motivou

`j`/`k` já andavam entre as barras (e a barra acendia), mas num
cronograma isso é pouco: não se via QUANDO cada barra acontece, `h`/`l`
não faziam nada, e a barra selecionada não dizia datas nem duração.

## O que mudou

- **Eixo de datas.** O núcleo põe uma parte `eixo` na frente das barras,
  com o começo e o fim da janela. A TUI desenha os rótulos (`03 ago`,
  como a janela) e uma régua com `┬` em cada marca, na mesma conta de
  coluna das barras. Marca por semana até 10 semanas, por mês até dois
  anos, por ano acima. O eixo tem linha, mas o cursor passa por cima
  dele (`cursor_passa_por_cima`).
- **`h`/`l` no tempo.** `j`/`k` seguem a ordem do arquivo; `h`/`l` andam
  pela ordem em que as barras ACONTECEM (`tela::barra_ao_lado`: começo,
  e empatado o começo, a mais curta). Contagem vale.
- **Detalhe.** Cada barra leva uma parte `detalhe` escondida
  (`03/08/2026 → 10/08/2026 · 8 dias · #infra`). Com o cursor numa parte
  que tem detalhe, ele aparece no pé da caixa do embed, junto do título
  — o que a janela mostra no `title` e no modal. O mecanismo é genérico,
  e o calendário usa em seguida.
- **Gaveta.** Item sem data vai pra `Sem data (N)` depois das barras, a
  mesma forma da gaveta do calendário.

## Critérios de aceite

- [x] Eixo com marcas alinhadas ao começo das barras
- [x] Enter no cronograma pula o eixo; `k` não volta pra ele
- [x] `h`/`l` na ordem do tempo, parando nas pontas
- [x] Detalhe da barra selecionada no pé da caixa, e só com o cursor nela
- [x] Item sem data na gaveta

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
