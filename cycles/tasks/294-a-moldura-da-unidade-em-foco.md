---
id: "294"
titulo: "A moldura da unidade em foco"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["293"]
estima_min: 120
---

# 294 — A moldura da unidade em foco

## Como isto virou o que é

Eu propus uma RÉGUA à esquerda de cada unidade — transparente, azul no
foco, com override pro embed. Implementei, mostrei, e a pessoa mandou um
desenho do que queria: **moldura**, caixa em volta da unidade e dos
filhos dela.

O desenho estava certo e a minha proposta não. Numa árvore a pergunta
não é só "que linha estou" — é "até onde vai o que estou olhando". Um
`##` com nove itens embaixo é uma coisa só, e é a moldura que diz isso.
Uma régua marca posição; ela não marca extensão.

## E o colapsável saiu de toda unidade

`▾`/`▸` em todo nível ensinava uma coisa que a janela não faz: lá uma
lista de markdown é uma lista, e quem tem botão de recolher é o callout,
porque ele É um objeto.

A seta ficou só no estado FECHADO — aí ela não é anúncio, é aviso de que
tem coisa escondida.

## E a lista aberta não desenha a própria linha

O `·` sozinho dentro da própria moldura era ruído. A caixa já diz onde a
lista começa e acaba, e na janela também não existe "linha da lista":
existe a lista, com os itens dentro.

Fechada ela volta, porque aí é a única coisa que representa o escondido.

## Critérios de aceite

- [x] A unidade em foco ganha caixa em volta dela e dos filhos
- [x] Sem foco no conteúdo, não há moldura
- [x] Unidade comum não anuncia dobra
- [x] Lista aberta não desenha a própria linha
- [x] Lista fechada volta a desenhar, com a contagem e a seta

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo clippy -p anotadinho-tui --all-targets
```
