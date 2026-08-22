---
title: Ciclo 120 — Grafo visual de backlinks
type: ciclo
ciclo: "120"
status: concluida
date: 2026-08-08
prioridade: baixa
depende_de: ["087", "088"]
tags:
- ciclo
---

# Ciclo 120 — Grafo visual de backlinks

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Grafo visual de backlinks

## Objetivo

Backlinks existem (ciclo 088) mas só como lista textual on-demand por
página. Novo `page_type: graph` (página inteira) mostra todas as
páginas do vault como nós e wikilinks como arestas, num SVG simples —
ajuda a navegar um vault de specs/decisões que já está crescendo.

## Critérios de aceite

- [x] Novo `page_type` "graph", dispatch em `page_view.rs` (mesmo
      padrão de `kanban`/`calendar`/`tags`), listado em `KNOWN_TYPES`
      do painel de propriedades
- [x] Componente novo `ui/src/components/graph_view.rs`: varre todas
      as páginas por `[[wikilink]]` (`wikilink::extract_titles`, novo,
      extraído da mesma lógica de `linkify`), monta nós (uma página =
      um nó, rótulo = título) e arestas (um wikilink resolvido = uma
      aresta)
- [x] Layout simples — nós num círculo (`2πi/n` por índice), SEM física
      de force-directed; arestas como `<line>` SVG entre os nós
- [x] Clicar um nó abre a página (mesmo `on_page_selected` usado em
      todo o resto do app)
- [x] Vault grande: cálculo é O(n) leituras + regex por página,
      aceitável pro tamanho atual; limitação documentada no docstring
      do componente
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [x] Validação ao vivo via MCP `tauri`: criei página `type: graph` no
      vault real, 24 nós (todas as páginas) + 2 arestas renderizaram
      corretamente, clicar um nó ("missao") abriu a página numa aba
      nova

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Layout force-directed/físico de verdade — círculo simples é
  suficiente pra v1; layout melhor é um ciclo futuro se o vault
  crescer a ponto do círculo ficar ilegível
- Zoom/pan interativo — SVG estático que cabe no viewport
- Filtrar o grafo por pasta/tag — mostra o vault inteiro sempre

## Notas

Não reaproveitou o painel de Backlinks de `editor.rs` diretamente
(que usa `search_content`/FTS5 como "grep" de `[[Título desta
página]]` — funciona bem pra UMA página, mas pra montar TODAS as
arestas do vault de uma vez seria N buscas separadas). Em vez disso,
`extract_titles` (novo, `wikilink.rs`) extrai a lista de wikilinks de
um texto — mesma lógica de scan de `linkify` mas coletando em vez de
substituir — e o `GraphView` faz `list_pages` + `read_page` de cada
uma, resolvendo cada wikilink por título (case-insensitive, mesmo
critério de `on_wysiwyg_click`). Isso deixa duas formas de achar
conexões no código (`search_content` pro painel por página,
`extract_titles` pro grafo do vault inteiro) — aceitável, resolvem
problemas de granularidade diferentes.

## Resultado

# Ciclo 120 - done

## Resumo

Novo `page_type: graph` — grafo visual das conexões entre páginas via
wikilinks. Nós num layout de círculo simples (sem física
force-directed), arestas SVG, clicar um nó navega pra página. Antes,
conexões só eram visíveis uma página de cada vez (painel de
Backlinks, ciclo 088).

## Arquivos criados/modificados

- `ui/src/wikilink.rs` — `extract_titles`, 4 testes
- `ui/src/components/graph_view.rs` (novo) — componente completo
- `ui/src/components/mod.rs` — registra o módulo
- `ui/src/components/page_view.rs` — dispatch `"graph"` →
  `GraphView`
- `ui/src/components/properties_panel.rs` — `"graph"` em `KNOWN_TYPES`
- `ui/src/styles/main.css` — `.graph-view*`

## Testes

`cargo test --workspace`: 111. `cd ui && cargo test --lib`: 79. Total 190.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: criei uma página `type: graph` no
vault real — 24 nós (todas as páginas), 2 arestas, layout em círculo
renderizou corretamente; clicar o nó "missao" abriu a página numa aba
nova. Página de teste removida do vault depois.

## Notas

Ciclo puramente frontend — sem mudança em `crates/*`/backend, sem
precisar reiniciar o processo de dev (hot-reload do `trunk serve`
cobriu tudo).

Próximo (último desta rodada): embed de PDF em frame scrollável (121).
