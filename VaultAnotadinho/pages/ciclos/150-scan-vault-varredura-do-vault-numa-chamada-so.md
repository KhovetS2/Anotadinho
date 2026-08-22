---
title: "Ciclo 150 — scan_vault: varredura do vault numa chamada só"
type: ciclo
ciclo: "150"
status: concluida
date: 2026-08-19
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 150 — scan_vault: varredura do vault numa chamada só

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

# Ciclo 150 - done

## Resumo

Varredura única do vault no backend, no lugar do padrão N+1 que todo
consumidor de "vault inteiro" usava (`list_pages()` + um `read_page()`
por página, cada um atravessando a ponte WASM↔Tauri com o arquivo
completo). `PageIndexEntry` carrega frontmatter YAML e properties
`chave:: valor` unificados, tags, alvos de wikilink e tags de dentro
dos embeds.

Migrados: grafo de backlinks, calendário em modo vault e a agregação de
tags. No VaultAnotadinho (24 páginas) a varredura inteira leva 7ms
numa chamada, contra 25 travessias antes.

## Arquivos criados/modificados

- `crates/core/src/links.rs` (novo) — `extract_wikilink_raw` /
  `extract_wikilink_targets` + 6 testes
- `crates/core/src/index.rs` (novo) — `PageIndexEntry` + 8 testes
- `crates/core/src/lib.rs` — registra os módulos
- `crates/ipc/src/lib.rs` — `handle_scan_vault` + 2 testes
- `src-tauri/src/main.rs` — comando `scan_vault`
- `ui/src/api.rs` — `api::scan_vault`
- `ui/src/embed.rs` — `scan_vault_calendar_entries` e
  `scan_vault_tags` numa chamada
- `ui/src/components/graph_view.rs` — arestas de `entry.wikilinks`

## Testes adicionados

- `links`: ordem, duplicatas, fence de código, alias/âncora, colchete
  aninhado, vazio
- `index`: frontmatter tipado + extra, properties do corpo, conflito
  YAML×property, título de fallback, YAML inválido, wikilinks únicos,
  tags de embed, campo inexistente
- `ipc`: varredura de vault com páginas e journals; vault vazio

## Problemas encontrados

- `PageIndexEntry` foi pro `core` e não pro `ipc` como a task dizia: o
  motor de consulta do ciclo 154 precisa dele testável sem Tauri.
- O vault mistura frontmatter YAML (`status:`) com property de corpo
  (`date::`) — o calendário lia a segunda forma. `properties` unifica
  as duas, com o YAML ganhando no conflito (é o que o painel de
  propriedades edita).
- `scan_vault_tags` precisava do corpo, não só de metadado. Em vez de
  deixar o último N+1 de pé, o parse de embed foi pro backend
  (`embed_tags`) — possível porque o ciclo 149 pôs `embed` no core.
- O app em execução precisou ser reiniciado pra validação ao vivo: o
  comando Tauri é novo, o WASM recarrega sozinho mas o binário não.

## Notas para próximos ciclos

- 154 (embed de query) tem a fonte de dados pronta: `api::scan_vault` +
  `PageIndexEntry::field()`.
- `properties` é `BTreeMap<String, String>` de propósito: consulta
  compara texto, e datas `YYYY-MM-DD` ordenam certo como string.
