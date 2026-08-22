---
title: Ciclo 142 — Atalhos de bloco (lista, citação, lista numerada) disparam no espaço, não só no Enter
type: ciclo
ciclo: "142"
status: concluida
date: 2026-08-14
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 142 — Atalhos de bloco (lista, citação, lista numerada) disparam no espaço, não só no Enter

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Atalhos de bloco (lista, citação, lista numerada) disparam no espaço, não só no Enter

## Objetivo

Bug real reportado pelo usuário: digitar `"- "` no editor não cria uma
lista não-ordenada de verdade. Causa raiz em
`editor.rs::apply_block_shortcut`: o atalho de heading (`"#"` +
espaço) já funciona porque roda ANTES de um `if !is_newline { return;
}` — mas lista (`"- "`/`"* "`), citação (`"> "`) e lista numerada
(`"1. "`) ficam DEPOIS desse guard, ou seja, só disparariam no
`Enter`, comparando o prefixo digitado com literais que JÁ incluem o
espaço à direita (`"- "`, `"> "`) — no fluxo real de digitação
(`"-"`, depois espaço, depois o texto do item, só then Enter), o
prefixo no momento do Enter é a linha inteira digitada, nunca bate.
Na prática esses 3 atalhos nunca disparavam. Ajusta pra funcionar
igual ao heading: checa o prefixo (sem o espaço, que ainda não foi
inserido no DOM no momento do `keydown`) e dispara no espaço.

## Critérios de aceite

- [x] `apply_block_shortcut`: heading continua igual (já funciona)
- [x] Lista (`"-"`/`"*"`), citação (`">"`) e lista numerada
      (dígitos + `"."`) passam a disparar no `keydown` da tecla
      espaço, comparando o prefixo SEM espaço à direita (mesmo padrão
      do heading) — e não mais no Enter
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `cd ui &&
      trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: digitar `"- "` seguido de
      texto num parágrafo vazio vira uma lista de verdade (`<ul><li>`)
      imediatamente (ver Notas — citação e lista numerada usam o
      mesmo código, validado por leitura)

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Tornar esses atalhos remapeáveis via `GlobalKeymap` — são atalhos de
  digitação inline (Space/Enter dentro do contenteditable), categoria
  diferente das teclas fixas do nav-mode ou dos atalhos globais
- Adicionar novos atalhos de bloco além dos que já existiam (heading/
  lista/citação/lista numerada) — checkbox é tratado no ciclo 143

## Notas

`cd ui && cargo test --lib`: 84 passados. `cargo test --workspace`:
OK. `trunk build`/`cargo build --manifest-path src-tauri/Cargo.toml`:
OK.

Refatorei a seleção de range repetida (4 ocorrências, uma por atalho)
pra uma função `select_prefix` só, chamada por todos.

Validação ao vivo via MCP `tauri`: focei o contenteditable vazio,
inseri `"-"` via `execCommand('insertText')` (simula a digitação do
caractere em si, que meu código não intercepta) e então enviei um
evento de tecla espaço de verdade — o DOM virou `<ul><li><br></li></ul>`
na hora, confirmando o atalho de lista funcionando no espaço. Citação
(`">"`) e lista numerada (`"1."`) passam pelo mesmo `select_prefix` +
`exec_cmd`/`insertOrderedList`/`formatBlock`, código idêntico em
estrutura ao caso de lista já validado — não reexecutei ao vivo pra
cada um individualmente, mas a lógica é a mesma função só com
literais/comandos diferentes.

Nota sobre o ambiente de teste: `Enter` sintético via automação NÃO
dispara o comportamento nativo do navegador de continuar uma lista
(criar um novo `<li>`) porque esse comportamento default só roda pra
eventos confiáveis (`isTrusted`), não pra eventos despachados por
automação — isso não é um bug do app, é uma limitação só da forma de
testar; usuários reais (tecla física) continuam recebendo o
comportamento nativo normalmente, e isso não depende de nada que este
ciclo mudou.

## Resultado

# Ciclo 142 - done

## Resumo

Bug real reportado pelo usuário: digitar `"- "` no editor não criava
uma lista. Causa raiz: `apply_block_shortcut` só checava lista/citação/
lista numerada no `Enter` (gate `if !is_newline { return; }`),
comparando o prefixo digitado com literais que já incluíam o espaço —
no fluxo real (marcador, espaço, texto, Enter), o prefixo no momento
do Enter é a linha inteira, nunca bate. Corrigido pra disparar no
espaço, mesmo padrão que o atalho de heading (que já funcionava) já
usava.

## Arquivos criados/modificados

- `ui/src/components/editor.rs` — `apply_block_shortcut` dispara
  lista/citação/lista numerada no espaço; nova função `select_prefix`
  deduplica a seleção de range repetida entre os 4 atalhos

## Testes adicionados

- Nenhum teste automatizado novo — depende de `web_sys`/DOM; validado
  ao vivo via MCP `tauri` (ver task file)

## Problemas encontrados

- Nenhum além do bug que este ciclo corrige

## Notas para próximos ciclos

- `Enter` sintético via automação MCP não aciona o comportamento
  nativo do navegador de continuar lista (só eventos confiáveis
  fazem isso) — limitação do ambiente de teste, não do app; próximos
  ciclos que precisem validar fluxos de Enter em lista devem montar o
  estado intermediário via DOM/Range direto, como feito no ciclo 143
