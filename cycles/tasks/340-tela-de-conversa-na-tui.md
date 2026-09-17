---
id: "340"
titulo: "A tela de conversa com o agente na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["339"]
---

# 340 — A tela de conversa na TUI

Referência: prints do DOM vivo da `ConversaView` e da `CommandPalette` da
janela (tirados pelo MCP bridge no dev server, voltando à página que
estava aberta).

Página com `type: conversa` abre como conversa (`app/conversa.rs`):

- topo com título, o agente (ϟ nome) e "N anexo(s)", anexos em pílulas;
- mensagens em caixas — as suas à direita tingidas de azul, as do agente
  à esquerda na superfície —, cabeçalho `VOCÊ · quando`, markdown desenhado
  como a página e quebrado na caixa;
- "⠋ pensando há 12s" com as últimas linhas da saída enquanto roda, e o
  erro em vermelho;
- campo embaixo (com cursor, `Ctrl+J` quebra linha) e "Enviar ↵" / "Parar".

Teclas: `j`/`k`/`G` passam pelas mensagens; `i` escreve (aí `q` e `:` são
texto); `Enter` envia; `Esc` sai do campo; `Enter` numa resposta abre
"virar spec / proposta / execução / copiar" (execução confirma); `y`
copia; `Ctrl+X` interrompe. Na barra de comandos: anexar página, tirar
anexo, interromper, trocar agente.

Por baixo:

- `agente.rs`: `Trabalho` roda o agente num processo filho com as peças do
  núcleo (`Adaptador::montar_args`, `LeitorStream`, timeout, cancelamento),
  e entrega a resposta uma vez.
- o `main` guarda as execuções por conversa (sair da tela não para o
  agente), grava a pergunta antes de disparar, monta o prompt com histórico
  e anexos (`conversa::montar_prompt`), grava a resposta ao acabar e relê a
  conversa aberta; anexos regravam o `contexto:` do frontmatter.
- núcleo: `conversa::reescrever_contexto` e `conversa::duracao_legivel`,
  que a janela passou a usar também.
- o agente vem das preferências (`Personalizar…` / `Trocar agente…`); sem
  escolha, o padrão do núcleo.

## O que não entrou

O seletor de prompt padrão e a configuração avançada do agente (binário,
pastas extras) — hoje só os presets.
