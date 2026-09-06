---
id: "278"
titulo: "Passo 4b: uma fonte só para a política do bloco"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["276"]
estima_min: 120
---

# 278 — Uma fonte só para a política do bloco

## O que é, dentro do passo 4

O passo 4 é a INVERSÃO: a árvore deixa de correr em paralelo e vira a
fonte da verdade. O ciclo 276 fez a metade da GRAVAÇÃO — a árvore decide
o que mudou ao salvar.

Esta é a metade da NAVEGAÇÃO, e começa pelo que é barato e sem volta: a
política. Quem recebe texto, quem comporta filhos, quem é grupo. Hoje
isso é respondido em dois lugares.

## A divergência, achada antes de mexer

`aceita_texto()` no editor mantinha uma lista de tags própria dizendo
que `ul`/`ol` **recebem texto**. O núcleo diz o contrário desde o ciclo
261: `Lista` é `Politica::GRUPO`, e desde o 273 o `<ul>` no DOM é
`contenteditable="false"`.

Duas respostas contrárias para a mesma pergunta. Não causava estrago
porque nenhum chamador de `insert_element_at_cursor` insere lista —
todos inserem figura, imagem ou marcador de embed. Sorte, não desenho:
no dia em que alguém colar uma lista por ali, o cursor pousa num
contêiner morto e a digitação some.

## O que fazer

`Tipo::da_tag(tag)` e `Tipo::tag()` no núcleo, e o editor pergunta:

- `aceita_texto()` → `da_tag(tag).politica().aceita_texto`
- `marcar_blocos()` → grupo é `aceita_filhos && !aceita_texto`, não
  `tag == "ul" || tag == "ol"`
- a regra do Enter → mesma pergunta

`table` continua na mão, declarado: tabela não é bloco do modelo (no
Anotadinho é embed), e o `<table>` que chega no editor vem de HTML
colado.

## Critérios de aceite

- [x] A política tem UMA fonte, e é o núcleo
- [x] A ponte tag↔tipo fecha o ciclo, com teste
- [x] A caixa da tag não importa (`tagName` do DOM é maiúsculo)
- [x] Tag desconhecida não vira bloco por descuido
- [x] A suíte não muda de resultado (255/255)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
