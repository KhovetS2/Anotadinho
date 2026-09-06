---
id: "293"
titulo: "O embed vira cartão no terminal"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["292"]
estima_min: 180
---

# 293 — O embed vira cartão

## A pergunta que motivou

"O que impede desenhar o embed no TUI como é na GUI?"

Nada de fundamental: caixa, pílula e botão são desenháveis em células.
O que impedia era estrutural, e estava no meu código.

## O que impedia

A tela era **uma linha por unidade**, e a conta entre cursor e rolagem é
feita em índice de linha visível. Uma caixa precisa de linhas
decorativas — a borda de baixo —, e cada uma desalinharia essa conta.
Foi por isso que a régua do h2 no ciclo 292 saiu como sublinhado.

## A mudança de base

`Linha` ganha `enfeite: bool`. Linha decorativa ocupa espaço, carrega o
caminho do dono (pra sumir junto quando ele dobra) e **não é destino**:
cursor, rolagem e `G` a ignoram.

Com isso a caixa fica possível — e a régua de verdade, e qualquer outra
decoração.

## O que foi desenhado

- **caixa**: o rótulo abre, um `└────` até a borda fecha. Só quando há
  conteúdo à vista: emoldurar uma linha só é enfeite sem função;
- **trilha do fluxo**: a fileira de etapas com a atual entre colchetes,
  como as pílulas da janela. "Concluída" sozinho não diz de onde veio
  nem pra onde vai.

## Critérios de aceite

- [x] Enfeite ocupa linha e não é destino
- [x] A caixa fecha depois do conteúdo e antes do bloco seguinte
- [x] Não sobra fecho depois do embed
- [x] Embed sem conteúdo não ganha caixa
- [x] A trilha do fluxo mostra a fileira com a atual destacada
- [x] A suíte da janela não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
