---
id: "110"
titulo: "CLI headless anotadinho-cli"
status: done
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# CLI headless anotadinho-cli

## Objetivo

Maior gap encontrado na avaliação de prontidão pro agent-os: hoje TUDO
que não é ler/escrever um arquivo `.md` direto (buscar, listar
páginas, exportar em massa, criar a partir de template) só existe
atrás da janela Tauri — `src-tauri/src/main.rs` só registra
`#[tauri::command]`, sem CLI/HTTP/stdio. `crates/vault` e
`crates/search` já são libs Rust puras, sem dependência de Tauri —
esse ciclo só precisa de um binário fino em cima delas. Isso dá pra um
agente (Claude Code, outro processo) rodar `anotadinho-cli <comando>`
num terminal comum e ler/buscar/exportar o vault sem precisar da GUI
aberta.

## Critérios de aceite

- [x] Novo crate binário `crates/cli` (nome do pacote: `anotadinho-cli`,
      adicionado ao workspace em `Cargo.toml` raiz), reaproveitando
      `anotadinho-ipc` (que já reaproveita `anotadinho-vault`/
      `anotadinho-search`, zero dependência de Tauri) como dependência —
      sem duplicar lógica de I/O
- [x] Subcomandos (parseados com `clap`, já que o padrão do resto do
      workspace é bem tipado):
      - `anotadinho-cli --vault <path> list-pages` — lista páginas
        (path/título/seção), uma por linha ou `--json`
      - `anotadinho-cli --vault <path> read <page_path>` — imprime o
        conteúdo bruto do `.md` no stdout
      - `anotadinho-cli --vault <path> search <query>` — busca FTS5,
        imprime path + trecho por página encontrada
      - `anotadinho-cli --vault <path> export [--folder <folder>]` —
        dump concatenado (mesmo formato do `export_folder`/`export_vault`
        do ciclo 101), imprime no stdout (permite `> arquivo.md` ou pipe
        pra outro processo/agente)
      - `anotadinho-cli --vault <path> list-templates` — lista templates
      - `anotadinho-cli --vault <path> new-from-template <template_path>
        <título>` — cria página a partir de template, imprime o path
        criado
- [x] `--json` nos comandos de listagem (`list-pages`/`search`/
      `list-templates`), pra consumo programático por outro processo/
      agente, além do formato humano (TSV) default
- [x] Erros vão pro stderr (`erro: <msg>`) com exit code 1
- [x] `--help` do próprio clap documenta os subcomandos (doc comments
      nos campos do enum `Command` viram a ajuda automaticamente)
- [x] `cargo build --workspace`, `cargo test --workspace` passam com o
      novo crate incluído

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo run -p anotadinho-cli -- list-pages --vault VaultAnotadinho
cargo run -p anotadinho-cli -- search --vault VaultAnotadinho "anotadinho"
cargo run -p anotadinho-cli -- export --vault VaultAnotadinho --folder pages | head
```

## Não-objetivos

- Servidor HTTP/MCP dedicado — CLI simples primeiro, um servidor é
  outro ciclo se o uso mostrar que compensa
- Comandos de escrita além de `new-from-template` (ex: editar
  frontmatter via CLI, mover página) — um agente já escreve `.md` e
  frontmatter direto no filesystem sem precisar de comando dedicado;
  os únicos comandos de escrita que valem a pena são os que têm lógica
  não-trivial (slug único, substituição de template)
- Empacotamento/distribuição (instalar globalmente, `cargo install`) —
  fica acessível via `cargo run -p anotadinho-cli --` por enquanto

## Notas

Descoberta ao implementar: nem precisou tocar `crates/vault` direto —
`crates/ipc` já expõe `handle_list_pages`/`handle_read_page`/
`handle_search_content`/`handle_export_folder`/`handle_list_templates`/
`handle_create_page_from_template` como funções Rust puras
(`Result<T, String>`, `vault_path: String`), a mesma camada que o app
Tauri usa — e `anotadinho-ipc` não tem NENHUMA dependência de Tauri no
seu `Cargo.toml`. Então o CLI é literalmente só `clap` + chamar esses
handlers + formatar saída; zero lógica de negócio duplicada ou nova.

Testado manualmente contra `VaultAnotadinho` (todos os 6 subcomandos)
e com 8 testes de integração (`crates/cli/tests/cli.rs`, via
`assert_cmd` chamando o binário de verdade) cobrindo sucesso e o
caminho de erro (`read` de página inexistente → stderr + exit 1).

`--vault` e `--json` são argumentos do parser raiz (não `global`,
já que só há um nível de subcomando — `global = true` nesse caso
conflita com `required` no clap 4.6, erro só visível em runtime via
debug assert).
