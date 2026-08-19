---
id: "150"
titulo: "scan_vault: varredura do vault numa chamada só"
status: done
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

- [x] `crates/core/src/links.rs` (novo): `extract_wikilink_targets(body)
      -> Vec<String>` — alvos de `[[...]]`, com `|` (alias) e `#`
      (âncora) recortados, sem duplicatas, ignorando ocorrências
      dentro de fence de código
- [x] `crates/core/src/index.rs` (novo): `PageIndexEntry { path,
      title, section, page_type, tags, properties, wikilinks,
      embed_tags }` + `from_content` + `field(name)`. Ficou no CORE
      (não no `ipc`) porque o motor de consulta do ciclo 154 precisa
      dele testável sem Tauri. `properties` unifica frontmatter YAML e
      `chave:: valor` do corpo — o vault usa as duas formas e quem
      consulta não deveria precisar saber de qual veio (YAML ganha no
      conflito)
- [x] `crates/ipc`: `handle_scan_vault(vault_path) ->
      Result<Vec<PageIndexEntry>, String>`, pulando em silêncio página
      que não puder ser lida
- [x] Comando Tauri `scan_vault` registrado em `src-tauri` e
      `api::scan_vault` no lado Yew
- [x] `graph_view.rs` migra pra 1 chamada (nós e arestas montados de
      `PageIndexEntry`)
- [x] `embed::scan_vault_calendar_entries` migra pra 1 chamada,
      lendo `date`/`time`/`end_date` de `properties`
- [x] `components/calendar.rs` migra pra 1 chamada (delega pra função
      acima)
- [x] `embed::scan_vault_tags` migra pra 1 chamada — o parse dos
      embeds passou pro backend (`PageIndexEntry::embed_tags`), fora
      do combinado inicial mas era o último N+1 restante
- [x] Testes de `extract_wikilink_targets` no core (alias, âncora,
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

`cargo test -p anotadinho-core`: 105 (91 + 14 novos).
`cargo test -p anotadinho-ipc`: 5 (3 + 2 novos). `cd ui && cargo test
--lib`: 26. `trunk build` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

Validação ao vivo (MCP `tauri`, app reiniciado pro comando novo entrar
no binário): `scan_vault` do VaultAnotadinho devolve as 24 páginas em
**7ms**, com wikilinks, tags, properties (`date::`/`status`/
`priority`) e `embed_tags` preenchidos. Página `grafo` continua
mostrando "24 páginas, 10 conexões"; página `calendario` continua
mostrando os mesmos 6 eventos — mesma saída de antes, uma chamada em
vez de 25.

A página `type: tags` não pôde ser conferida ao vivo (o vault de
exemplo não tem nenhuma), mas a fonte dela (`embed_tags`) foi conferida
na saída do `scan_vault`: `pages/exemplos-embeds.md` →
`["bug", "infra", "urgente"]`.

`ui/src/wikilink.rs` continua existindo e não muda: ele parseia com
posições pra renderizar/linkificar dentro do editor. O `links.rs` do
core resolve outro problema (só os alvos, fora do WASM, pro grafo e
pro CLI) — mesma sintaxe, usos diferentes.
