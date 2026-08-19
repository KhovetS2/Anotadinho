---
id: "150"
titulo: "scan_vault: varredura do vault numa chamada só"
status: pending
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# scan_vault: varredura do vault numa chamada só

## Objetivo

Todo lugar que precisa olhar o vault inteiro hoje faz N+1 chamadas de
IPC: `list_pages()` e depois um `read_page()` por página, cada um
atravessando a ponte WASM↔Rust com o conteúdo completo do arquivo.
Acontece em `graph_view.rs` (l.123/149), em
`embed::scan_vault_calendar_entries` e no calendário de página
inteira. Num vault de 200 páginas isso é 201 round-trips só pra
desenhar um grafo. Este ciclo cria uma varredura única no backend que
devolve só os metadados — e vira o pré-requisito de performance do
embed de query (154).

## Critérios de aceite

- [ ] `crates/core/src/links.rs` (novo): `extract_wikilink_targets(body)
      -> Vec<String>` — alvos de `[[...]]`, com `|` (alias) e `#`
      (âncora) recortados, sem duplicatas, ignorando ocorrências
      dentro de fence de código
- [ ] `crates/ipc`: `handle_scan_vault(vault_path) ->
      Result<Vec<PageIndexEntry>, String>` com
      `PageIndexEntry { path, title, section, frontmatter:
      BTreeMap<String, String>, tags: Vec<String>, page_type: String,
      wikilinks: Vec<String> }` — o frontmatter achatado em strings
      (valores escalares como estão; listas juntadas por `, `), que é
      o que query/grafo/calendário precisam
- [ ] Comando Tauri `scan_vault` registrado em `src-tauri` e
      `api::scan_vault` no lado Yew
- [ ] `graph_view.rs` migra pra 1 chamada (nós e arestas montados de
      `PageIndexEntry`)
- [ ] `embed::scan_vault_calendar_entries` migra pra 1 chamada,
      lendo `date`/`time`/`end_date` do frontmatter achatado
- [ ] `components/calendar.rs` migra pra 1 chamada
- [ ] Testes de `extract_wikilink_targets` no core (alias, âncora,
      duplicata, dentro de fence, linha sem link) e de
      `handle_scan_vault` em `crates/ipc` com vault temporário

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Cache/índice persistente (SQLite) — a varredura é feita na hora, sob
  demanda; a busca full-text já tem o FTS5 dela em `crates/search`
- Watcher invalidando o índice — fora de escopo, cada consumidor
  rechama quando monta
- Mudar `list_pages`/`read_page` existentes — continuam pra quem só
  precisa do conteúdo de uma página

## Notas

`ui/src/wikilink.rs` continua existindo e não muda: ele parseia com
posições pra renderizar/linkificar dentro do editor. O `links.rs` do
core resolve outro problema (só os alvos, fora do WASM, pro grafo e
pro CLI) — mesma sintaxe, usos diferentes.
