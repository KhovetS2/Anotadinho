---
id: "258"
titulo: "Fechar as specs que já estavam implementadas"
status: done
criado: 2026-09-05
autor: agente
prioridade: media
depende_de: ["255", "256", "257"]
estima_min: 40
---

# 258 — Fechar as specs

## O problema

Nove specs implementadas, nenhuma fechada. Três paradas em `em-revisao`
desde os ciclos 250–254, duas em `aprovada` com tudo pronto, uma
`concluida` com os critérios de aceite ainda desmarcados.

O efeito colateral disso é pior que a bagunça: **o estado da spec deixou
de significar alguma coisa**. `tema-configuravel` estava inteiramente
implementada com os seis critérios em branco, e `atalhos-do-dia-a-dia`
estava `concluida` com quatro em aberto. Quem consultasse o vault pra
saber o que falta encontraria uma lista errada nos dois sentidos.

## Critérios de aceite

- [x] Cada spec fechada diz ONDE cada requisito foi entregue, por ciclo
- [x] Os critérios de aceite só são marcados com cobertura verde
      apontável, nunca no olho
- [x] Divergência entre o que a spec pediu e o que foi entregue fica
      escrita na spec, não escondida
- [x] As duas specs `exemplo-*` ficam intactas

## As duas que NÃO fecharam, de propósito

`exemplo-busca-dentro-de-embed` (`in-progress`) e
`exemplo-exportar-nota-em-pdf` (`backlog`) não são backlog: são conteúdo
do vault de exemplo, e existem justamente pra o [[Painel]] mostrar o
fluxo completo com estados diferentes. Fechá-las esvaziaria o bloco "Em
andamento" que elas servem pra demonstrar.

## O que ficou registrado como divergência

- **`imagens-coladas-e-arrastadas`**: o arraste abre o modal em vez de
  inserir na hora (diverge da letra do RF1, e foi a resposta dada na
  seção de perguntas em aberto da própria spec); a referência gravada é
  `<figure>` e não `![](…)`.
- **`tema-configuravel`**: o critério "o snapshot passa em pelo menos
  dois temas" era impossível como estava — um tema existe pra mudar cor,
  então comparar cor entre temas reprova por definição. O que o snapshot
  confere na segunda passada é geometria.

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
