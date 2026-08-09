---
id: "136"
titulo: "Nav-mode: polimento e casos de borda"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Nav-mode: polimento e casos de borda

## Objetivo

Último ciclo do tema nav-mode (133-135): supressão do nav-mode
enquanto qualquer overlay estiver aberto (app-level e menus locais),
validação de coexistência com o vim mode, e (pedido adicional do
usuário nesta mesma conversa) um indicativo visual de profundidade —
gradiente azul→roxo no destaque de região e no badge, mais forte
quanto mais fundo a sessão de navegação está.

## Critérios de aceite

- [x] `app.rs`: `any_overlay_open` (pending_dialog/palette/cheatsheet/
      vim_settings/global_keymap_settings) suprime o nav-mode por
      completo enquanto QUALQUER overlay de nível de app estiver aberto
- [x] Guarda adicional `focus_is_nav_tracked` — Enter/Backspace/Escape
      só agem se `document.activeElement` ainda for algo com
      `data-nav-item` (setas se auto-curam sozinhas, ver Notas) —
      cobre os menus LOCAIS (⚙, popover de git, "⋯" do editor) que não
      são overlays do `app.rs`, então `any_overlay_open` não os vê
- [x] `header_bar.rs`/`editor.rs`: os 3 menus dropdown devolvem o foco
      pro próprio botão de abrir ao fechar via Escape (bug real
      encontrado durante a validação, ver Notas)
- [x] Indicativo visual de profundidade (pedido adicional): nova
      `nav_mode::depth_color_css(depth)` — degradê azul puro
      (profundidade 1) → roxo puro (profundidade 5+) via
      `color-mix()`, aplicado tanto no badge quanto no destaque de
      região via uma custom property CSS (`--nav-mode-depth-color`)
- [x] Validação de coexistência com o vim mode: `j`/`i`/Escape dentro
      do `.editor__wysiwyg` continuam intocados pelo nav-mode (guarda
      `is_text_input_target` já existente desde o ciclo 133 dá conta);
      nav-mode continua funcionando normalmente FORA do editor com o
      vim mode ligado ao mesmo tempo
- [x] `cd ui && cargo test --lib` (84, +3 testes de
      `depth_color_css`), `cargo test --workspace`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Revalidação end-to-end dos ciclos 133-135 juntos (delegate pro
      kanban continua funcionando depois das mudanças deste ciclo)

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum novo — ciclo de fechamento

## Notas

### Bug real encontrado e corrigido durante a validação

Testando a supressão com overlay aberto, descobri que `any_overlay_open`
sozinho NÃO bastava: os 3 menus dropdown (⚙, popover de git, "⋯" do
editor, ciclo 125) são estado LOCAL de cada componente
(`header_bar.rs`/`editor.rs`), nunca hoisted pro `app.rs` — então
abrir o menu ⚙ via nav-mode (Enter numa folha chama `.click()`) e
depois apertar Escape fazia DUAS coisas ao mesmo tempo: o listener de
Escape do próprio menu fechava ele, E o nav-mode (que continuava achando
que estava tudo normal) TAMBÉM subia um nível/saía da sessão — os dois
"ouvem" o mesmo Escape.

Corrigido com um sinal mais genérico (`focus_is_nav_tracked`): quando
um menu abre, o auto-foco do ciclo 125 já move `document.activeElement`
pra DENTRO do menu (não é mais o item que tem `data-nav-item`) — se o
foco atual não é algo que o nav-mode colocou lá, o Enter/Backspace/
Escape não é dele.

Isso revelou um SEGUNDO bug, mais sutil: fechar o menu (Escape) não
devolvia o foco pra lugar nenhum específico — caía em `<body>`. E
`<body>` é ANCESTRAL de `.app-root`, não descendente — eventos de
teclado só borbulham pra CIMA (target → ancestrais), nunca descem;
um keydown nascendo em `<body>` nunca alcança o listener em
`.app-root`, então o nav-mode ficava "preso" (badge continuava
mostrando a sessão ativa, mas nenhuma tecla mais fazia efeito, nem as
setas — que eu tinha desenhado pra se "auto-curar", mas isso só
funciona se o evento CHEGA no listener primeiro). Corrigido fazendo os
3 menus devolverem o foco pro próprio botão de abrir ao fechar via
Escape (`menu_toggle_ref`/`git_popover_toggle_ref`/
`editor_menu_toggle_ref`, novos) — resolve o bug em si E deixa a UX
mais correta de qualquer forma (padrão comum: fechar um popup devolve
o foco pra quem abriu ele).

### Indicativo visual de profundidade

Pedido adicional do usuário no meio da implementação deste ciclo.
`nav_mode::depth_color_css(depth: usize) -> String` é pura (testada
sem DOM): profundidade 1 = `var(--accent-blue)` puro, profundidade 5+
satura em `var(--accent-purple)` puro, meio-termo via
`color-mix(in srgb, var(--accent-blue) X%, var(--accent-purple) Y%)`
com X caindo 25 pontos por nível. Aplicado via uma ÚNICA custom
property CSS (`--nav-mode-depth-color`, setada inline em Rust) que
tanto o badge quanto `.nav-mode__region-active` consultam — sem
precisar de N classes CSS por profundidade.
