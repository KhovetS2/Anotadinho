---
id: "341"
titulo: "Prompt padrão e campo aceso na conversa da TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["340"]
---

# 341 — Prompt padrão e campo aceso na conversa

Referência: print do popover de prompt da janela (lista, campos
`{{variável}}` "Preencha antes de enviar", Visualizar, Fechar).

- `p` (ou `Ctrl+P` escrevendo, ou "Usar prompt padrão…" na barra) abre o
  seletor, embaixo à esquerda como o popover: "Nenhum — escrever do zero" e
  os prompts de `pages/prompts-default/` com `type: prompt`
  (`prompt_padrao::descobrir`).
- Escolher lê a página (pedido `CarregarPrompt`) e aplica como a janela: o
  rascunho vira o valor da primeira variável (blindado), sem variáveis o
  rascunho vai no fim, o `contexto:` do prompt entra nos anexos (e falha
  se a página não existe). Com variáveis, os campos ficam no seletor:
  `Tab` anda, digitar preenche e o campo da conversa mostra o molde com o
  que já há; `Ctrl+V` visualiza o texto final; `Enter` no último fecha.
- Enviar com marcador pendente é recusado, com o aviso e o seletor aberto;
  o rodapé do campo avisa enquanto falta. Mexer no texto montado vira
  mensagem livre; "Nenhum" devolve o rascunho de antes.
- O botão mostra o prompt em uso ("▤ Nome ⌃").
- Destaque do campo: escrevendo, a caixa ganha contorno inteiro de meio-bloco
  na cor de destaque e fundo elevado, e o rodapé vira "-- INSERÇÃO --" com
  as teclas; com o cursor no campo sem escrever, o contorno fica meio aceso.
