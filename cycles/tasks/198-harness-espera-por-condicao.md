---
id: "198"
titulo: "Harness: espera por condição em vez de relógio"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [197]
estima_min: 90
agente_alvo: claude-opus
---

# Harness: espera por condição

## Objetivo

A suíte levava 7min43s, e 115 desses segundos eram esperas FIXAS
(`PAUSA(1500)` depois do reload, `PAUSA(2200)` depois de abrir a
página). Tempo dimensionado pro pior caso e desperdiçado em todos os
outros. Suíte lenta é rodada com menos frequência, e suíte que não roda
não vale nada.

## Critérios de aceite

- [x] `recarregarEstavel` — recarrega e espera o documento ser TROCADO,
      não um tempo fixo.
- [x] `abrirPaginaEstavel` — abre e espera o conteúdo PARAR de mudar.
- [x] `esperarEstavel` — duas leituras iguais seguidas.
- [x] Os quatro arquivos de cenário usando os novos helpers.
- [x] Suíte inteira verde, e mais rápida.

## Comandos de validação

```bash
node scripts/uitest/run.mjs
```

## Resultado

**236.1s contra 462.8s** — 49% mais rápido, com os mesmos 85 cenários.
A bateria de digitação sozinha caiu de 95s pra 51s.

## Notas

**A corrida que quase passou despercebida:** a primeira versão do
`recarregarEstavel` esperava "a sidebar tem itens" logo depois de pedir
o reload. Isso passa NA HORA, porque o DOM antigo ainda está lá — e o
cenário clicava num nó prestes a ser destruído. Um marcador em `window`
resolve: ele só some quando o documento é realmente trocado.

O sintoma foi feio: a suíte inteira travou sem imprimir uma linha. A
lição é que espera por condição só é mais rápida se a condição
distinguir o estado NOVO do VELHO — senão ela não espera nada.
