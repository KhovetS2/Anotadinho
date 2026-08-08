---
id: "087"
titulo: "Wikilinks: parse render e popup no editor"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: ["086"]
estima_min: 120
agente_alvo: claude-sonnet
---

# Wikilinks: parse, render e popup no editor

## Objetivo

Segundo ciclo do conjunto grande (ver
`/home/elis/.claude/plans/jaunty-tinkering-beaver.md`). Sintaxe
`[[Título da Página]]` — resolvida por título, renderizada como link
normal, com popup de autocomplete ao digitar `[[` no editor e navegação
real ao clicar. Base necessária pro ciclo seguinte de backlinks.

## Critérios de aceite

- [x] `ui/src/wikilink.rs` novo: `encode_title`/`decode_title`
      (percent-encoding mínimo, roundtrip), `linkify` (regex-free,
      preserva blocos de código), com testes
- [x] `markdown_render::render` aplica `linkify` antes do
      `pulldown_cmark` — `[[Título]]` vira `<a href="anotadinho://page/...">`
      de verdade na visualização
- [x] `html_to_md.rs`: caso `"a"` serializa de volta pra `[[Título]]`
      quando o `href` é do scheme interno (usa o texto visível, não o
      href — sobrevive a edição de texto dentro do link)
- [x] Editor: digitar `[[consulta` abre popup de autocomplete (mesmo
      mecanismo do menu `/` — `find_wikilink_context`, scroll-into-view,
      fechar ao clicar fora, ArrowUp/Down/Enter/Escape) listando páginas
      do vault filtradas por título; selecionar insere o link via
      `insert_element_at_cursor(el, false)` (inline-safe)
- [x] Clicar um wikilink já renderizado navega pra página de verdade
      (`on_page_selected`), resolvendo por título (case-insensitive,
      primeiro match em caso de ambiguidade — ver Notas)
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Desambiguação por picker quando há títulos duplicados — resolve pelo
  primeiro match (lista já vem ordenada por título); UI de escolha fica
  pra depois se virar problema real
- Backlinks (painel de "quem linka pra cá") — próximo ciclo, depende
  deste
- Renomear página atualizando wikilinks que apontam pra ela em outras
  páginas — os links guardam o TÍTULO, não o path, então renomear uma
  página quebra os wikilinks existentes (aponta pro título antigo, que
  não existe mais); consertar isso automaticamente é fora de escopo aqui

## Notas

Desvio da heurística de desambiguação descrita no plano original ("mais
recentemente editado"): `PageMeta` não tem timestamp de modificação, e
adicionar isso seria uma mudança de escopo maior (thread mtime por I/O →
IPC → API → struct em 3 lugares). Usei "primeiro match" (lista já vem
ordenada por título de `list_pages`) em vez disso — mais simples, ainda
resolve o caso comum (só há ambiguidade se o usuário duplicar título
entre pastas de propósito).

`insert_element_at_cursor(el, false)` (não quebra pra fora do bloco) —
igual imagem/asset, porque wikilink é conteúdo inline-safe (faz sentido
ficar dentro do parágrafo/item de lista onde foi digitado), diferente de
título/lista/citação/embed que precisam quebrar pra fora.

Validado ao vivo via MCP `tauri` na página `teste`: digitar `[[kan` abre
popup filtrado (`kanban`, `kanban-projeto`); selecionar insere
`<a href="anotadinho://page/kanban">kanban</a>` inline dentro do `<li>`;
clicar o link navega pro editor da página `kanban` (confirmado pelo
título mudando); salvar+reabrir a página `teste` confirma
`- [[kanban]]` no arquivo bruto e o link renderizado de volta
corretamente. `[[xyz` (título inexistente) mostra "Nenhuma página com
esse título" no popup; Escape fecha sem alterar o texto digitado.
Mudança de teste revertida em `VaultAnotadinho/pages/teste.md` antes de
fechar o ciclo.

Achado de ambiente (não bug de código, reafirma a nota do ciclo 086): o
bridge `webview_keyboard` (`action: "type"`) não funciona em elementos
`contenteditable` (só em `<input>`/`<textarea>`, porque tenta setar
`.value`) — usei `document.execCommand('insertText', ...)` via
`webview_execute_js` pra simular digitação real no editor durante a
validação.
