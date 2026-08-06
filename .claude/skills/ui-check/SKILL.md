---
name: ui-check
description: Valida visualmente uma mudança de UI no Anotadinho — garante que o app está rodando e usa os MCP servers (tauri/playwright) ou um fixture estático pra conferir o resultado.
---

# ui-check

Anotadinho é Tauri + Yew (WASM) servido via `trunk serve` numa janela
nativa. Uma aba de browser comum apontando pra `localhost:1420` NÃO
funciona pra testes funcionais: toda operação de dado (`abrir vault`,
`listar páginas`, `ler/gravar arquivo`) passa por
`window.__TAURI_INTERNALS__.invoke(...)` (ver `ui/src/api.rs`), que só
existe dentro do webview real do Tauri. Uma aba comum trava na tela
inicial. Leve isso em conta antes de escolher a estratégia abaixo.

## 1. Garanta que o dev server está de pé

```bash
ps aux | grep -E 'trunk serve|cargo-tauri' | grep -v grep
```

Se não estiver rodando, suba com `./scripts/dev.sh` (isso abre a janela
Tauri nativa — não é algo que rode em background silenciosamente, então
avise o usuário que uma janela vai abrir).

`trunk serve` faz hot-reload sozinho quando arquivos `.rs`/`.css` mudam —
normalmente não precisa reiniciar nada, só esperar o rebuild (`trunk build`
bem-sucedido já é sinal de que o reload vai acontecer).

## 2. Verifique se os MCP servers de UI estão disponíveis nesta sessão

```
ToolSearch("select:mcp__tauri")   # controla a janela Tauri real (tem a ponte IPC)
ToolSearch("select:mcp__playwright") # browser genérico
```

Esses servers estão configurados em `.opencode/opencode.json` (originalmente
pro opencode) e podem ter sido registrados no Claude Code via
`claude mcp add tauri -- npx -y @hypothesi/tauri-mcp-server` /
`claude mcp add playwright -- npx -y @playwright/mcp`. **MCP servers
registrados a meio de uma sessão só ficam disponíveis numa sessão nova** —
se `claude mcp list` já mostra os dois conectados mas o `ToolSearch` acima
não acha nada, é isso: avise o usuário que uma nova sessão (ou
`/mcp reload` se existir) é necessária, não fique tentando de novo.

## 3a. Se o MCP `tauri` estiver disponível

Use-o pra navegar até a página relevante na janela real (que tem a ponte
IPC) e tirar screenshot — esse é o único caminho pra validar fluxo
funcional completo (abrir vault, editar, embeds interativos, etc).

## 3b. Se só o MCP `playwright` (ou nenhum) estiver disponível

Não dá pra testar fluxo funcional. Ainda dá pra validar CSS puro: monte um
fixture HTML estático que carrega `ui/dist/main-*.css` +
`ui/dist/components-*.css` (rode `trunk build` antes pra garantir que o
dist está atualizado) com uma amostra de markup representativa (ex: copie
a estrutura de `.editor__wysiwyg` com uma lista, um heading, um embed) e
abra esse fixture via playwright (ou peça pro usuário abrir manualmente).
Deixe claro no relatório que isso valida só CSS, não o comportamento real
do app.

## 4. Reporte

Screenshot (ou descrição do que foi visto) + confirmação específica do que
estava sendo verificado (ex: "recuo dos bullets: ok, ~24px" em vez de só
"parece bom").
