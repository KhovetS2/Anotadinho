---
id: "308"
titulo: "Um badge por tag no multiselect, e todo embed como caixa preenchida"
status: done
criado: 2026-09-16
autor: agente
prioridade: alta
depende_de: ["307"]
estima_min: 150
---

# 308 — Badge por tag e todo embed preenchido

## O que motivou

Dois pedidos juntos: o multiselect da tabela com um badge por tag (o
305 mostrava "urgente, bug" como um texto só, na cor da primeira), e a
caixa preenchida que o callout ganhou no 307 aplicada aos outros
embeds.

## Badge por tag

Na janela, `.task-table__tags` é uma fileira de `.badge`, cada tag na
cor da SUA posição nas opções. A célula multiselect virou um galho em
linha (`tags`) com uma `tag--info`/`tag--success`… por valor; o texto da
célula continua sendo o valor inteiro, como no arquivo. As tags são
dado de desenho (`fica_fora_da_tela`): quem as desenha é a linha da
tabela, uma pílula ao lado da outra.

Select e multiselect saem em PÍLULA (`Tema::pilula`: texto na cor do
badge, fundo com 15% dela, um espaço de cada lado — o `padding` do
`.badge`). A largura da coluna passa a contar as pílulas.

## Todo embed preenchido

`Tema::fundo_da_regiao` devolve fundo pra todo papel de embed, não só
pro callout: 8% da cor do tipo (a do 304) sobre `--bg-surface`. A
moldura, a lateral e o preenchimento já eram genéricos desde o 307.

A moldura do FOCO continua só borda: ela diz até onde vai o que se
olha, não que aquilo é um objeto.

Uma coisa só apareceu na tela: dentro do fundo tingido, `--border` fica
quase da cor do fundo, e as bordas dos dias do calendário e o `│` da
tabela sumiram. Viraram `Realce::Grade` (texto apagado misturado à
superfície), com teste de distância contra o fundo de todo embed.

## Critérios de aceite

- [x] Multiselect com uma pílula por tag, cada uma na sua cor
- [x] Select em pílula; colunas alinhadas contando as pílulas
- [x] Todo embed com fundo tingido pela cor do tipo, fundos distintos
- [x] Sem faixa escura em volta dos cartões e botões dentro da caixa
- [x] Grade do calendário e da tabela visível sobre o fundo
- [x] A moldura do foco continua só borda

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
