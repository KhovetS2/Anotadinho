---
title: Ciclo 123 — Foco visivel em toda a aplicacao
type: ciclo
ciclo: "123"
status: concluida
date: 2026-08-09
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 123 — Foco visivel em toda a aplicacao

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Foco visível em toda a aplicação

## Objetivo

Auditoria encontrou: `main.css` não tem NENHUMA regra `:focus-visible`
em lugar nenhum. Navegar por Tab pela aplicação inteira hoje não
mostra NENHUM indicador visual em NENHUM botão — a única exceção é
`.sidebar-item--nav-active`, uma classe customizada por JS (modo de
seta do ciclo 106), não `:focus` de verdade. Pior: `.editor__wysiwyg`,
`.editor__textarea`, `.task-table__number-input`,
`.task-table__text-input` têm `outline: none` sem NENHUM substituto —
foco fica literalmente invisível nesses. Base fundacional pra tudo que
vem depois nesse tema (paridade de teclado só é útil se dá pra ver
onde o foco está).

## Critérios de aceite

- [x] Regra global em `main.css`: `:focus-visible` aplicada a
      `button`, `a`, `input`, `select`, `textarea`, `[tabindex]` —
      `outline: 2px solid var(--accent-blue)` + `outline-offset: 2px`
- [x] Remove os `outline: none` sem substituto — `.editor__wysiwyg`
      ganha `box-shadow: inset` sutil (outline grande seria ruído a
      cada tecla, o cursor piscando já sinaliza foco durante a
      digitação); `.task-table__number-input`/`.task-table__text-input`
      ganham realce de fundo (`background: var(--bg-elevated)`), já
      que são inputs sem borda dentro de célula de tabela —
      `.editor__textarea` confirmado como CSS morto (nenhum componente
      usa essa classe, o editor usa só `.editor__wysiwyg` desde que
      virou contenteditable) — não mexido, fora do escopo limpar CSS
      morto não relacionado
- [x] `.table-select-menu__input:focus`/`.calendar-grid__view-select:focus`
      deixados intocados (já tinham indicador próprio, risco baixo de
      sobrepor mal com a regra `:focus-visible` genérica)
- [x] Validação: regras confirmadas presentes e sintaticamente válidas
      via `document.styleSheets` no app rodando (ver Notas — validação
      visual completa não foi possível pela limitação do canal de
      automação, documentado)
- [x] `trunk build`, `cd ui && cargo test --lib` passam

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Reformular o design visual do anel de foco por elemento (cor/
  espessura customizada por tipo de componente) — uma regra genérica
  consistente é o suficiente pra v1
- Focus trap dentro de modais — isso é o ciclo 124
- Foco visível dentro do SVG do grafo (nós não são focáveis ainda de
  jeito nenhum) — isso é o ciclo 126, que primeiro precisa tornar os
  nós focáveis antes de estilizar o foco deles

## Notas

`:focus-visible` (não `:focus`) é o seletor certo aqui — a maioria dos
navegadores modernos (incluindo WebKitGTK) já implementa a heurística
de só aplicar quando a navegação foi por teclado, então não precisa de
JS extra pra distinguir "cliquei com mouse" de "naveguei com Tab".

**Limitação de validação encontrada**: a heurística de `:focus-visible`
exige eventos de teclado CONFIÁVEIS (`isTrusted: true`) — nem
`dispatchEvent(new KeyboardEvent(...))` via `webview_execute_js` nem
`webview_keyboard press Tab` (driver MCP) fizeram `element.matches(
':focus-visible')` retornar `true` no elemento focado. Confirmei que
as 4 regras CSS estão presentes e sintaticamente corretas no
stylesheet carregado (`document.styleSheets`), e a feature é padrão
bem suportado (sem prefixo) no WebKitGTK usado pelo Tauri — mas não
consegui confirmar visualmente com um "Tab" de verdade através da
automação disponível. Fica registrado como limitação conhecida de
ferramental pra ciclos futuros deste tema: validação de
`:focus-visible` especificamente vai precisar de teste manual humano
ou de uma ferramenta de automação com input de teclado genuinamente
confiável.

## Resultado

# Ciclo 123 - done

## Resumo

Regra global `:focus-visible` em `main.css` — antes deste ciclo não
existia NENHUM indicador visual de foco em NENHUM elemento da
aplicação (exceto a classe customizada da navegação de setas da
sidebar). `outline: none` sem substituto em 3 lugares reais (editor,
inputs de tabela) corrigido com indicadores adequados ao contexto de
cada um.

## Arquivos criados/modificados

- `ui/src/styles/main.css` — regra global `:focus-visible`, indicador
  em `.editor__wysiwyg`, `.task-table__number-input`,
  `.task-table__text-input`

## Testes

`cd ui && cargo test --lib`: 79. `trunk build`: OK.

## Notas

Ciclo puramente CSS. Validação visual completa de `:focus-visible`
não foi possível via automação (a heurística exige input de teclado
confiável, `isTrusted: true` — nem `dispatchEvent` nem o driver MCP
de teclado disparam isso) — confirmado apenas que as regras estão
presentes e sintaticamente corretas no stylesheet carregado.
Documentado como limitação conhecida no arquivo de task.

Próximo: Modal com foco automático/trap/Escape (124) — a correção
que resolve o bug relatado (não dava pra escolher template via
teclado).
