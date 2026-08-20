---
id: "179"
titulo: "Foco perdido ao fechar modal no nav-mode"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: []
estima_min: 45
agente_alvo: claude-opus-5
---

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
