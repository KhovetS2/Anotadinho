---
title: Ciclo 124 — Modal com foco automatico trap de Tab e Escape
type: ciclo
ciclo: "124"
status: concluida
date: 2026-08-09
prioridade: alta
depende_de: ["123"]
tags:
- ciclo
---

# Ciclo 124 — Modal com foco automatico trap de Tab e Escape

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Modal com foco automático, trap de Tab e Escape

## Objetivo

Bug reportado: abrir o diálogo "Escolher template" (`PendingDialog::
Select`, usado ao criar página) via atalho de teclado (Ctrl+N) e não
conseguir selecionar as opções pelo teclado. Causa raiz: `Modal`
(`ui/src/components/modal.rs`) — usado por Prompt/Confirm/Select/
Propriedades/Histórico — nunca move o foco pra dentro de si quando
abre, não tem handler de Escape (só fecha clicando fora), e não trapeia
Tab (some pra fora do modal). `PendingDialog::Prompt` só funciona hoje
por acaso, porque o `<input>` tem `autofocus` manual — `Select`,
`Confirm` e o resto não têm nada.

## Critérios de aceite

- [x] `Modal` foca automaticamente o primeiro elemento focável dentro
      do CORPO (`.modal__body`, não o botão "✕" do cabeçalho) assim
      que `open` vira `true` — via `use_effect_with(props.open, ...)`
      + `NodeRef`, sem precisar que cada consumidor (`dialog_host.rs`
      etc) implemente `autofocus` manualmente
- [x] Tab/Shift+Tab dentro do modal fica preso (cicla do último pro
      primeiro elemento focável — incluindo o "✕" e os botões de
      `.modal__actions` — e vice-versa) — não escapa pro resto da
      página
- [x] Escape fecha o modal (chama `props.on_close`) de qualquer lugar
      dentro dele, não só clicando fora
- [x] `PendingDialog::Select`: navegação por setas NÃO implementada —
      confiando em Tab/Shift+Tab nativo entre os `<button>` (que já
      funciona, ver Notas), conforme a flexibilidade prevista neste
      próprio critério
- [x] `autofocus` manual do `PendingDialog::Prompt` continua
      funcionando, sem conflito com o autofoco novo do `Modal`
      (confirmado: ambos convergem pro mesmo `<input>`, chamar
      `.focus()` duas vezes no mesmo elemento é inofensivo)
- [x] `cd ui && cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam
- [x] Validação ao vivo via MCP `tauri`: abrir "Nova página" — modal
      "Escolher template" já nasce com foco em "Página em branco"
      (primeiro item) sem nenhum Tab manual; testei o trap de Tab
      diretamente (focar o último elemento real do modal, "Cancelar",
      e dar Tab → vai pro "✕"; focar "✕" e dar Shift+Tab → vai pro
      "Cancelar"); Escape fecha sem escolher

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Reimplementar Confirm/Alert com navegação de setas — só têm 1-2
  botões, Tab already resolve
- Focus trap em popovers/menus dropdown fora do `Modal` (⚙, git
  status, ⋯ do editor) — esses não usam o componente `Modal`, são
  `<div>` próprios; ciclo 125 cuida deles separadamente
- Restaurar o foco pro elemento que abriu o modal, ao fechar — bom
  ter, mas não é o bloqueante reportado; fica pra depois se notarem falta

## Notas

`command_palette.rs` já tem o padrão de referência pronto (índice
ativo + `ArrowDown`/`ArrowUp`/`Enter`/`Escape`, ver
`command_palette.rs:74` e `:188-201`) — usado como referência
conceitual, mas não copiado direto: como o `PendingDialog::Select`
usa `<button>` reais (diferente da paleta, que usa um `<input>` de
busca + lista renderizada), o foco nativo do navegador entre os
botões já cobre "mover entre opções" sem precisar de um índice ativo
próprio.

O foco automático do `Modal` é o item que desbloqueia TUDO que usa
`Modal` de uma vez (Prompt/Confirm/Select/Propriedades/Histórico) —
prioridade alta porque foi a correção mais barata com o maior
alcance deste tema.

**Descoberta importante durante a validação — limitação do canal de
automação, não bug do app**: `webview_keyboard press Tab`/`Enter` via
MCP dispara o evento DOM normalmente (confirmado com um listener
nativo — o evento chega e bubbleia certinho), mas NÃO aciona
comportamentos NATIVOS do navegador que dependem de input "confiável"
(`isTrusted`): avançar o foco pro próximo elemento em Tab no meio da
lista, ou ativar um `<button>` focado com Enter. Isso inicialmente
pareceu que o trap de Tab não funcionava — só depois de isolar o
teste (focar programaticamente o ÚLTIMO elemento real do modal via
`.focus()`, que NÃO depende de input confiável, e só então disparar
Tab) ficou claro que o código estava certo o tempo todo; o erro era
eu ter assumido que a última opção da lista (`spec`) era o último
elemento focável, quando na verdade é o botão "Cancelar" que vem
depois. Mesma classe de limitação já documentada no ciclo 123 pro
`:focus-visible`. Fica reforçado como padrão de validação pros
próximos ciclos deste tema: `.focus()` programático funciona sempre;
comportamento NATIVO do navegador em resposta a tecla simulada, não —
testar o código PRÓPRIO isolando do que é comportamento nativo não
escrito por mim.

## Resultado

# Ciclo 124 - done

## Resumo

`Modal` (usado por Prompt/Confirm/Select/Propriedades/Histórico) ganha
foco automático no primeiro elemento do corpo, trap de Tab/Shift+Tab
(cicla dentro do modal), e Escape fecha de qualquer lugar dentro dele.
Corrige o bug reportado: o diálogo "Escolher template" nunca recebia
foco sozinho, então não dava pra navegar as opções só com teclado.

## Arquivos criados/modificados

- `ui/src/components/modal.rs` — reescrito: `FOCUSABLE_SELECTOR`,
  auto-foco via `use_effect_with(props.open, ...)`, `on_keydown` com
  Escape + trap de Tab

## Testes

`cd ui && cargo test --lib`: 79. `trunk build` +
`cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: abrir "Nova página" — modal
"Escolher template" nasce com foco em "Página em branco" sem nenhum
Tab manual; trap testado nas duas direções (Tab do último elemento
volta pro primeiro, Shift+Tab do primeiro volta pro último); Escape
fecha sem escolher; Prompt (nome da pasta) continua auto-focando o
input, sem regressão.

## Notas

Limitação de automação encontrada e documentada (ver arquivo de
task): `webview_keyboard` via MCP não aciona comportamento NATIVO do
navegador ligado a input "confiável" (avanço de Tab no meio da lista,
Enter ativa botão focado) — só `.focus()` programático funciona de
forma confiável pra validação automatizada. Não afeta usuário real
com teclado físico.

Próximo: navegação por teclado nos menus dropdown (125).
