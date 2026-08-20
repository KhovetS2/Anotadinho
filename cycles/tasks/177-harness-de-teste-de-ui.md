---
id: "177"
titulo: "Harness de teste de UI"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: []
estima_min: 120
agente_alvo: claude-opus-5
---

# Harness de teste de UI

## Objetivo

Pedido do usuário. Quase todo bug do editor que escapou dos testes
existentes é comportamento de DOM, invisível pro `cargo test`: texto
duplicado dentro de `contenteditable` (076), arraste que não commitava
(155), Escape fechando a página junto com o modal (161), toolbar
cobrindo o controle do embed (166). Todos foram achados no braço,
clicando — e um deles (161) passou 40 ciclos despercebido.

Este ciclo transforma essa validação manual em suíte roteirizada contra
o app DE VERDADE.

## Critérios de aceite

- [x] Runner sem dependência nova: Node 22+ (o `WebSocket` já é global)
      falando com o MCP Bridge que o app expõe na 9223
- [x] `node scripts/uitest/run.mjs` roda tudo; com argumento, filtra por
      nome do cenário
- [x] Sai com código != 0 se algum cenário falhar
- [x] Mensagem clara quando o app não está de pé, dizendo o que fazer
- [x] Cada cenário cria e apaga a própria página de rascunho
      (`pages/__uitest.md`) — nunca toca no conteúdo real do vault
- [x] Cobre as regressões que já aconteceram: menu `/` com os 9 tipos
      (148), round-trip do callout (151), arraste do cronograma (155),
      Escape no modal (161), sobreposição da toolbar (166), teclado nos
      embeds (165), recarga ao mudar no disco (173)
- [x] Suíte inteira roda em menos de 30s
- [x] Documentado no `AGENTS.md`, na validação por ciclo

## Comandos de validação

```bash
./scripts/dev.sh              # num terminal
node scripts/uitest/run.mjs   # noutro
```

## Não-objetivos

- Rodar em CI sem display: exige o app gráfico de pé. Serve como suíte
  local antes de fechar ciclo — que é onde os bugs aparecem
- Substituir os testes de unidade: eles cobrem lógica pura (parse,
  consulta, aritmética de data), e são muito mais rápidos
- Screenshot comparado pixel a pixel (frágil demais pro valor)

## Notas

Protocolo do bridge descoberto por sondagem:
`{"id","command":"execute_js","args":{"script"}}` →
`{"data","success","error"}`.

Uma armadilha do próprio harness virou comentário no código: o
`mousedown` só marca o estado, e os listeners de `mousemove`/`mouseup`
entram no efeito da renderização seguinte. Disparar o arraste inteiro
num bloco de JS só acusa uma regressão que não existe — o cenário do
cronograma manda em duas chamadas, com pausa.
