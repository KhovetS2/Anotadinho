---
id: "123"
titulo: "Foco visivel em toda a aplicacao"
status: pending
criado: 2026-08-09
autor: humano
prioridade: alta
depende_de: []
estima_min: 75
agente_alvo: claude-sonnet
---

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

- [ ] Regra global em `main.css`: `:focus-visible` (não `:focus` —
      evita anel de foco em clique de mouse, só aparece navegando por
      teclado) com contorno visível e consistente (`outline: 2px solid
      var(--accent-blue)` + `outline-offset`, mesma cor já usada no
      indicador customizado da sidebar) aplicada a `button`, `a`,
      `input`, `select`, `textarea`, `[tabindex]` — um seletor
      genérico o suficiente pra cobrir tudo sem precisar listar classe
      por classe
- [ ] Remove os `outline: none` sem substituto (`.editor__wysiwyg`,
      `.editor__textarea`, `.task-table__number-input`,
      `.task-table__text-input`) — ou troca por um indicador
      equivalente (borda/box-shadow) se `outline` quebrar o layout
      nesses elementos especificamente
- [ ] `.table-select-menu__input:focus`/`.calendar-grid__view-select:focus`
      (que já têm `border-color` como indicador) continuam funcionando,
      sem duplicar com a regra global de forma que fique estranho
      visualmente (dois indicadores sobrepostos)
- [ ] Validação ao vivo via MCP `tauri`: tab pela sidebar, header,
      editor, confirma visualmente (via `webview_get_styles` ou
      inspeção de `outline`/`box-shadow` computado no elemento com
      foco) que cada parada do Tab tem um indicador visível
- [ ] `trunk build`, `cd ui && cargo test --lib` passam (mudança só de
      CSS, não deveria quebrar nada, mas roda mesmo assim)

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
