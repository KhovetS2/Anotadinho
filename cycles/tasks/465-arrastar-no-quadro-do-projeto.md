---
id: "465"
titulo: "Módulo Projeto — arrastar o cartão entre as colunas do quadro, respeitando as etapas"
status: done
criado: 2026-10-21
autor: agente
prioridade: media
depende_de: ["464"]
estima_min: 120
---

# 465 — Arrastar no quadro do projeto

Pedido da pessoa: "o quadro do projeto não aceita arrastar como no
kanban?". No 460 o arrasto ficou de fora (os botões cobriam). Agora ele
entra, com a diferença que o quadro tem e o kanban não: a regra de
etapa.

## Critérios de aceite

- [x] Pegar um cartão (botão esquerdo, fora dos botões de mover) acende
      só as colunas pra onde a PESSOA pode levá-lo
      (`proximas_para(Humano)`). As outras ficam marcadas como
      proibidas.
- [x] Soltar numa coluna permitida move a tarefa pelo mesmo caminho dos
      botões (`mudar_etapa_tarefa`). Soltar em `executar` põe na fila.
- [x] Soltar numa coluna proibida, ou na própria, não grava nada.
      `executando` e `revisao` nunca aceitam, porque são do app.
- [x] Soltar fora do quadro solta o arrasto (o listener de `mouseup` na
      janela, como no kanban), e o estado nunca fica preso.
- [x] Clicar no título continua abrindo a tarefa, e os botões continuam
      funcionando.
- [x] Harness: soltar no permitido grava, no proibido não grava, e em
      Executando não grava.

## Decisões

- **Eventos de mouse, não o arrasto nativo do HTML5.** É o caminho do
  `inline_kanban`: no WebKitGTK o nativo é instável, e o app roda com
  `dragDropEnabled: false`.
- **Sem posição dentro da coluna.** A ordem é a do número da tarefa, e a
  da fila é prioridade e número. Uma posição arrastada que nada
  respeitasse seria mentira na tela.
- **Só a janela.** A TUI não tem mouse para arrastar, e lá ficam os
  números.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs "projeto:"
node scripts/uitest/run.mjs
```
