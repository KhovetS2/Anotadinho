---
title: Agente e execução
date: 2026-08-24
dominio: agente
tags:
- padrao
---
# Agente e execução

## Quando se aplica

Qualquer mudança no caminho que executa um agente externo: adaptador,
prompt, acompanhamento, ou o que ele pode alcançar.

## As regras

1. **Sem shell.** `Command::new(binario).args(...)`, com o prompt como
   UM argumento. Aspas, quebras e `$(...)` no prompt são texto.
2. **A configuração vem das preferências**, nunca do conteúdo de uma
   página. Uma nota que chegue de terceiro não escolhe o que roda.
3. **Conteúdo do vault entra no prompt BLINDADO**, entre marcadores e
   precedido do aviso de que é dado, não instrução.
4. **A execução não bloqueia a tela.** O processo é do backend; a tela
   pergunta como está. Sair da página não mata nem perde a resposta.
5. **Quem grava a resposta é o backend**, porque a tela pode não estar
   lá.
6. **Uma execução por conversa.** É o que garante que nunca há dois
   escritores no mesmo arquivo.
7. **O motivo da falha vem do STREAM**, não do stderr — o stderr quase
   sempre é ruído de inicialização.
8. **A pasta de trabalho é escolha da pessoa.** É a escolha dela que
   autoriza a escrita ali.
9. **Timeout generoso (30 min) com botão de interromper.** Limite curto
   mata trabalho legítimo no meio.

## Por que existe

- **202** — o desenho sem shell é o que fecha injeção pelo prompt.
- **213** — timeout de 3 min matava o planejamento no meio; a
  requisição vivia no componente e sumia ao trocar de página.
- **214** — `claude -p` segura a saída até o fim; sem `stream-json` não
  há sinal de vida.
- **216** — rodar no vault deixava o agente sem ver o código E com
  escrita nas notas: o pior dos dois mundos.
- **219** — a conta bateu o limite e a tela mostrou "Reading additional
  input from stdin...".
