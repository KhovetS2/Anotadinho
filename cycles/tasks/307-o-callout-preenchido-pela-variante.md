---
id: "307"
titulo: "O callout como caixa preenchida, na cor da variante"
status: done
criado: 2026-09-16
autor: agente
prioridade: alta
depende_de: ["306"]
estima_min: 150
---

# 307 — O callout preenchido pela variante

## O que motivou

Adiado desde o 304: a moldura do callout tinha uma cor só (`warning`)
pra toda variante, e só a BORDA era colorida. O pedido: cor por
variante e o fundo da caixa todo preenchido, não só a borda.

## O que a janela faz

`.callout` tem `background: color-mix(in srgb, var(--callout-accent)
8%, var(--bg-surface))`, `border-left: 3px solid` no acento, e um
cabeçalho com o ícone da variante no acento e o título em negrito. O
acento é `--accent-blue` (info), `--success`, `--warning`, `--error` e
`--accent-purple` (tip).

## A variante chega no desenho

A moldura só conhecia o NOME do embed. `partes_do_embed` passa a pôr na
frente do corpo uma parte `variante` (dado de desenho, escondida da
tela) e, quando há, o `titulo`. `regiao()` lê a variante na árvore e
pinta a caixa com `Realce::Callout*`.

A variante é IRMÃ do título e do corpo — diferente do cronograma e do
calendário, em que todos os filhos escondidos eram irmãos entre si.
Isso pedia duas coisas em `tela`: `andar` passa por cima de parte
escondida na mesma direção (Enter pousa no título, `k` no título não
volta pra variante), e `contagem` não conta parte escondida ("2 items"
num callout de um parágrafo mentiria).

## A caixa preenchida

`Tema::fundo_da_regiao` devolve o fundo da janela (8% do acento sobre
`--bg-surface`) só pro callout. Com fundo, `moldura` desenha meio-bloco
(`▗▄▖`/`▝▀▘`, a caixa do botão do 302) na cor do fundo, e `emoldurar`
pinta todo trecho sem fundo próprio e completa até a borda; a lateral
esquerda é meia célula no acento, o `border-left` da janela.

O título ganha o ícone da variante (`ℹ ✔ ⚠ ✖ ✦`, uma célula cada —
emoji ocupa duas e desalinharia a caixa). Sem título, o rótulo
`[callout]` sai na cor da variante.

## Critérios de aceite

- [x] Cada variante pinta a caixa com a sua cor
- [x] O fundo cobre a caixa inteira, até a borda direita
- [x] Lateral esquerda na cor da variante
- [x] Título com ícone da variante; sem `[callout]` quando há título
- [x] A variante não aparece na tela nem recebe o cursor
- [x] As outras molduras continuam só borda

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
