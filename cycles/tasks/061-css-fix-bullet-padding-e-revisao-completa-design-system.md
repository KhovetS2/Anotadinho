---
id: "061"
titulo: "CSS fix bullet padding e revisao completa design system"
status: done
criado: 2026-08-06
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
