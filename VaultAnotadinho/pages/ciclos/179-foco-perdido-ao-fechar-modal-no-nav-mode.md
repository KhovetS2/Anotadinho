---
title: Ciclo 179 — Foco perdido ao fechar modal no nav-mode
type: ciclo
ciclo: "179"
status: concluida
date: 2026-08-21
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 179 — Foco perdido ao fechar modal no nav-mode

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Foco perdido ao fechar modal no nav-mode

## Objetivo

Relatado pelo usuário: navegando pelos botões do embed de ações, Enter
no "+ ação" abre o modal; Escape fecha; **daí em diante as setas não
andam mais**.

Duas causas, encontradas ao reproduzir:

1. O `Modal` (ciclo 124) rouba o foco ao abrir e não devolve ao fechar.
   O foco caía num `<div>` sem `data-nav-item`, e o motor de navegação
   exige um item rastreado (guarda do ciclo 136) — as setas ficavam
   mudas. Pior: o destaque continuava aceso no botão, então PARECIA que
   ainda dava pra andar.
2. `items_in_group` incluía itens `display: none` — os controles que só
   aparecem no hover (configurar/remover botão). `focus()` neles não faz
   nada, então a navegação empacava naquele índice.

## Critérios de aceite

- [x] Fechar um modal devolve o foco pro elemento que o abriu
- [x] Elemento que sumiu da página enquanto o modal estava aberto não
      quebra nada (verifica se ainda existe antes de focar)
- [x] Item escondido (`display: none`) não entra na lista de navegação
- [x] Item com `opacity: 0` CONTINUA entrando: ele reaparece ao receber
      foco (`:focus-within`), então é alvo legítimo
- [x] Cenário no harness com o fluxo exato do relato: entrar no embed,
      andar até "+ ação", Enter, Escape, seta

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs "fechar modal"
```

## Não-objetivos

- Mudar o auto-foco do ciclo 124 (o modal deve mesmo receber o foco ao
  abrir)
- Restaurar o foco em popups que não são `Modal` (menus já devolvem o
  foco pro botão que os abriu, desde o ciclo 161)

## Notas

O primeiro sintoma que apareceu foi o cenário do harness não conseguir
chegar no "+ ação" — foi assim que a segunda causa (itens escondidos)
apareceu. O relato do usuário e o teste falhando eram o mesmo bug visto
de dois ângulos.

## Resultado

# Ciclo 179 - done

## Resumo

Abrir um modal pelo teclado e fechá-lo com Escape deixava o nav-mode
mudo: o foco caía num `<div>` sem `data-nav-item` e o motor exige item
rastreado. O `Modal` passou a devolver o foco pra quem o abriu.

Junto veio uma segunda causa achada ao reproduzir: itens
`display: none` (controles que só aparecem no hover) entravam na lista
de navegação e travavam a sequência, porque `focus()` neles não faz
nada.

## Arquivos criados/modificados

- `ui/src/components/modal.rs` — guarda o elemento focado ao abrir e o
  restaura no cleanup
- `ui/src/nav_mode.rs` — `items_in_group` filtra item sem área na tela
- `scripts/uitest/cenarios.mjs` — cenário com o fluxo do relato

## Testes adicionados

- Cenário de harness: entra no embed, anda até "+ ação", Enter, Escape,
  confere que o foco voltou pro botão e que a seta volta a andar

## Problemas encontrados

- `opacity: 0` NÃO pode ser filtrado junto com `display: none`: a
  toolbar do embed e as barras de hover usam opacidade e reaparecem no
  `:focus-within` — filtrá-las tornaria vários controles inalcançáveis
  pelo teclado.

## Notas para próximos ciclos

- O harness pegou a segunda causa antes do relato chegar nela: o
  cenário não conseguia alcançar o botão.
