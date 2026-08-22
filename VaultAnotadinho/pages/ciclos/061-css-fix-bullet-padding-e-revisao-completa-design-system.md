---
title: Ciclo 061 — CSS fix bullet padding e revisao completa design system
type: ciclo
ciclo: "061"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 061 — CSS fix bullet padding e revisao completa design system

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CSS fix bullet padding e revisao completa design system

## Objetivo

Corrigir o bug relatado de bullets de lista grudados na lateral (regra de
"block hover indicator" sobrescrevendo o padding-left de `ul`/`ol`) e fazer
uma revisão completa do CSS: remover blocos órfãos/quebrados, CSS morto,
realinhar cores antigas (paleta pré-rebrand) com os tokens atuais, e
atualizar `docs/design-system.md`/remover `theme.rs` morto. Usar o AppFlowy
real como referência visual, já que o Anotadinho é uma reimplementação dele.

## Critérios de aceite

- [x] `ul`/`ol` fora do seletor do "block hover indicator" em `main.css` — recuo de lista volta a `1.5rem`
- [x] Blocos CSS órfãos removidos (`main.css:48-75`, `main.css:312-321`)
- [x] CSS morto removido: `.editor__delete/export/save`, duplicata de `.sidebar-search .input`, e também o
      bloco inteiro do split-pane preview (`.editor__split`/`.editor__preview-content`, já marcado
      "deprecated" no próprio comentário e sem nenhum uso em `.rs`)
- [x] `rgba()` da paleta antiga realinhados — trocados por `color-mix(in srgb, var(--token) X%, transparent)`
      em vez de recravar hex novo, pra não repetir o mesmo tipo de drift no futuro
- [x] `theme.rs` removido (morto, nunca importado) e `docs/design-system.md` atualizado pros tokens/arquivos reais
- [x] `.tab-bar`/`.tab-bar__tab`: os ~11 `!important` não tinham nenhuma regra concorrente no CSS
      (confirmado via grep) — removidos. Os 2 `!important` que sobraram no arquivo
      (`.tab-bar__tab-close:hover`, `.header-menu__item`) têm conflito de especificidade real
      (documentado com comentário inline em cada um)
- [x] Padding/margin em px cru trocado por `--sp-*` onde havia equivalente exato (6 ocorrências:
      `.header-bar`, `.header-menu`, `.header-menu__item`, `.sidebar-section__header`,
      `.editor__statusbar`, `.slash-menu__list`). Mistos (ex: `6px 16px`, onde só um lado bate
      com um token) foram deixados como estavam — não vale forçar.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cd ui && trunk build
```

## Não-objetivos

- Reescrever o layout/estrutura dos componentes — só CSS, sem mudar `.rs`
- Migrar `components.css` de volta pra `main.css` (a divisão em 2 arquivos é legítima; o certo é atualizar a doc, não desfazer o arquivo)

## Notas

Ver `/home/elis/.claude/plans/jaunty-tinkering-beaver.md` (Workstream C).
Clone raso do AppFlowy (`github.com/AppFlowy-IO/AppFlowy`) só como referência
de paleta/spacing — é Flutter, então é inspiração de design, não código a
copiar.

## Resultado

# Ciclo 061 - done

## Resumo

Corrigido o bug relatado (bullets de lista grudados na lateral: a regra de
"block hover indicator" incluía `ul`/`ol` no mesmo seletor de `p`/headings,
sobrescrevendo o `padding-left: 1.5rem` da lista pra `8px`). Aproveitado pra
fazer a revisão completa de CSS pedida: 2 blocos de CSS quebrados/órfãos
(declarações sem seletor, sobra de edições incompletas), CSS morto
(botões duplicados do editor, preview split-pane já marcado deprecated),
paleta antiga sobrevivendo em `rgba()` cravados, módulo `theme.rs` morto e
desatualizado, `docs/design-system.md` com os valores de cor antigos, e
`!important` sem motivo real na `.tab-bar`.

## Arquivos criados/modificados

- `ui/src/styles/main.css` — fix do bug relatado, remoção de blocos órfãos/CSS morto,
  `rgba()` antigos trocados por `color-mix(in srgb, var(--token) X%, transparent)`,
  `!important` sem justificativa removidos (os 2 que sobraram têm comentário explicando o motivo),
  padding/margin convertidos pra `--sp-*` onde havia equivalente exato, CSS novo pra
  `.editor__wysiwyg-segments` (layout dos embeds inline do ciclo 060)
- `ui/src/styles/components.css` — mesmo tratamento de `rgba()` antigos
- `ui/src/theme.rs` — removido (morto, nunca importado, valores desatualizados)
- `ui/src/lib.rs` — remove `pub mod theme;`
- `docs/design-system.md` — tokens de cor atualizados pros valores reais, regra de
  "um único arquivo CSS" corrigida pra documentar a divisão real (main.css + components.css)

## Testes adicionados

Nenhum (CSS puro, sem lógica Rust nova). Validado via `trunk build` (confirma CSS
sintaticamente válido no pipeline real) e checagem de balanceamento de chaves.

## Problemas encontrados

- Nenhum. Não cheguei a clonar o AppFlowy como referência (mencionado nas notas da task) —
  a paleta atual já estava internamente consistente, só desalinhada em alguns `rgba()`
  cravados; não havia necessidade de uma referência externa pra essa correção pontual.

## Notas para próximos ciclos

- `.editor__wysiwyg-segments` e as classes `.embed-kanban`/`.embed-calendar`/`.embed-table`
  (novas, do ciclo 060) reaproveitam classes já existentes (`.kanban`, `.calendar`,
  `.task-table__table`) — não duplicaram CSS.
