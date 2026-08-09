---
id: "134"
titulo: "Nav-mode: grupos de página inteira e cheatsheet"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["133"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Nav-mode: grupos de página inteira e cheatsheet

## Objetivo

Segundo ciclo do nav-mode: o delegate `"editor"` (`.app-main-panel`,
ciclo 133) só sabia focar `.editor__wysiwyg` — páginas tipadas
(kanban/calendário/tabela/grafo) não tinham NENHUM jeito de receber
foco via nav-mode. Marca o conteúdo desses 4 componentes com
`data-nav-content-root`, generaliza o delegate pra cair nesse marcador
como fallback, e documenta as teclas fixas do nav-mode na cheatsheet.

## Critérios de aceite

- [x] `data-nav-content-root="true"` no elemento raiz de conteúdo de
      `kanban.rs`, `calendar.rs`, `task_table.rs`, `graph_view.rs`
      (só no branch com conteúdo de verdade — não nos estados de
      loading/vazio, que não têm nada focável mesmo)
- [x] `app.rs`: delegate `"editor"` tenta `.editor__wysiwyg` primeiro
      (comportamento inalterado pra página de texto normal); se não
      achar, cai pro primeiro `[tabindex="0"]` dentro de
      `[data-nav-content-root]` — reaproveita o Enter/Espaço que
      esses 4 componentes já tinham dos ciclos 126/127, nav-mode só
      entrega o foco inicial
- [x] `cheatsheet_modal.rs`: nova seção "Modo de navegação" com as 5
      teclas fixas da sessão (setas iniciar/mover, Enter, Backspace,
      Escape) — o atalho de LIGAR a capacidade (`toggle_nav_mode`) já
      aparecia sozinho na seção "Globais" (lê `labeled_fields()` ao
      vivo, sem precisar de entrada manual)
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: delegate `"editor"` numa
      página kanban (foco cai no primeiro card) e numa página de grafo
      (foco cai no primeiro nó SVG); cheatsheet mostra a seção nova

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- `tags_page.rs`/`assets_page.rs` não ganharam o marcador — não
  citados no plano original deste ciclo; extensão trivial (1 linha
  cada) se algum dia fizer falta
- Teclado nos embeds inline — ciclo 135
- Supressão com overlays abertos, coexistência com vim mode — ciclo 136

## Notas

### Bug real encontrado e corrigido durante a validação

`nav_mode::focus_item` fazia `el.dyn_ref::<web_sys::HtmlElement>()` e,
se `None`, simplesmente não fazia nada — funcionava pra card/linha/
chip (todos HTML), mas os nós do grafo são `<g>` SVG, um ramo
COMPLETAMENTE separado da hierarquia de elementos do DOM
(`SvgElement`, não `HtmlElement`). O cast falhava silenciosamente, e
`.focus()` nunca era chamado — sintoma ao vivo: delegate pra "editor"
numa página de grafo fazia a sessão encerrar (badge sumia, confirmando
que o código do delegate rodou) mas o foco ficava preso no
`.app-main-panel`, sem mover pra nenhum nó.

Corrigido com um segundo braço em `focus_item` tentando
`dyn_ref::<web_sys::SvgElement>()` (que TAMBÉM tem `.focus()` — SVG
implementa o mesmo mixin `HTMLOrSVGElement` da spec) quando o cast pra
`HtmlElement` falha. Precisou adicionar a feature `"SvgElement"` no
`web-sys` do `ui/Cargo.toml` (não estava habilitada — primeira vez que
o projeto chama `.focus()` num elemento SVG via Rust; antes disso os
nós do grafo só eram alcançados via Tab nativo do navegador, que não
passa por `web_sys` nenhum).

Esse era um bug latente que NENHUM código anterior do projeto tinha
disparado — cycles 126/127 deram `tabindex`+`onkeydown` aos nós do
grafo mas nunca precisaram chamar `.focus()` neles programaticamente
(o usuário sempre chegava lá via Tab nativo). O nav-mode foi o
primeiro código a tentar isso, e por isso o achou.
