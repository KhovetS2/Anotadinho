---
id: "210"
titulo: "Aviso de proposta pendente e execução da proposta aprovada"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [204, 209]
estima_min: 150
agente_alvo: claude-opus
---

# Aviso de pendente e execução

## Objetivo

Itens 3 e 4 da spec aprovada [[Uso agêntico do Anotadinho no dia a
dia]], fechando o ciclo completo: spec → proposta → execução.

## Critérios de aceite

- [x] Indicador no cabeçalho com a contagem de propostas pendentes.
- [x] Conta propostas de QUALQUER canal — UI, CLI ou MCP.
- [x] Clicar leva pra revisão, criando a página se não existir: aviso
      não pode levar a lugar nenhum.
- [x] O aviso some quando a fila zera.
- [x] Proposta APROVADA oferece "Executar", com pergunta diferente da de
      planejar.
- [x] A pergunta de execução proíbe mudar a abordagem sem proposta nova.
- [x] "virar execução" nas ações da resposta.
- [x] 2 cenários de harness.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs
```

## As duas travas, e por que são diferentes

| Etapa | O que o modelo NÃO pode fazer |
|---|---|
| Planejar | propor requisito novo ou mudar escopo — isso é da spec |
| Executar | mudar a abordagem — se ela não serve, PARE e explique |

A segunda existe porque uma abordagem trocada no meio da execução é uma
proposta que ninguém revisou. Se ela for mesmo inviável, o caminho é uma
proposta nova, não um desvio silencioso.

## Bug achado ao testar

**O aviso ficava preso.** Ele só reconsultava quando a lista de páginas
mudava, e recusar uma proposta não mexe nela: a contagem continuava
mostrando 1 com a fila vazia. A tela de revisão passou a avisar que a
fila mudou.
