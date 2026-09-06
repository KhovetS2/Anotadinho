---
id: "283"
titulo: "Os embeds declaram seu conteúdo"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["281"]
estima_min: 180
---

# 283 — Os embeds declaram seu conteúdo

## A premissa errada que eu repeti

Eu disse, ao fechar o ciclo 282, que "faltam nove embeds declarar". Isso
vem do ciclo 267 — e o **268 já resolveu**. Está escrito no status dele:

> "nenhum embed precisou declarar nada. Os nove ganharam movimento sem
> uma linha de mudança."

O padrão geométrico serviu todos. A declaração de INTERESSE DE TECLA que
o 267 abriu só é necessária quando o embed quer o movimento pra si — o
calendário quer, porque anda por datas, não por geometria.

## A lacuna que sobra de verdade

`analisar` guarda todo embed como unidade **atômica sem filhos**. O
renderizador de terminal desce no atômico (`desce_no_atomico`, ciclo
266) e não acha nada: a página do porte CLI para na borda de cada embed.

## Conteúdo, não controle

Medi o DOM dos nove antes de escrever. Cada embed mistura duas coisas
como `[data-nav-item]`:

```
galeria: 16 itens navegáveis — 2 são as imagens
kanban:  cartão e "apagar cartão" são os dois itens
```

Só o primeiro grupo existe no markdown. Botão de apagar é cromo da GUI,
e um terminal desenharia o seu. A árvore descreve o conteúdo.

## O que é dinâmico não entra

As linhas de uma consulta vêm do vault, e `analisar` é função pura do
corpo da página. A consulta declara o que está ESCRITO (de onde), não o
resultado. O mesmo vale pro calendário em modo vault.

## Critérios de aceite

- [x] `Tipo::Parte` descreve uma parte interna sem mentir que é markdown
- [x] Os nove declaram seu conteúdo a partir do que o núcleo já parseia
- [x] O embed continua ATÔMICO — a navegação da GUI não muda
- [x] A ida e volta do vault continua fiel
- [x] O terminal desenha o conteúdo de um kanban
- [x] A suíte não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
