---
title: Ciclo 110 — CLI headless anotadinho-cli
type: ciclo
ciclo: "110"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 110 — CLI headless anotadinho-cli

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

# Ciclo 110 - done

## Resumo

Fecha o maior gap encontrado na avaliação de prontidão pro agent-os:
até este ciclo, tudo além de ler/escrever um `.md` direto exigia a
janela Tauri rodando. Novo crate `crates/cli` (binário
`anotadinho-cli`) expõe `list-pages`, `read`, `search`, `export`,
`list-templates` e `new-from-template` reaproveitando os handlers de
`anotadinho-ipc` (que já é Tauri-free) — dá pra um agente ler, buscar
e exportar o vault inteiro de um terminal comum, sem GUI.

## Arquivos criados/modificados

- `crates/cli/Cargo.toml` (novo) — `anotadinho-cli`, dependências:
  `anotadinho-ipc`, `clap` (derive), `serde`/`serde_json`
- `crates/cli/src/main.rs` (novo) — parser clap + 6 subcomandos
- `crates/cli/tests/cli.rs` (novo) — 8 testes de integração via
  `assert_cmd` contra o binário real
- `Cargo.toml` (raiz) — `crates/cli` adicionado a `members`

## Testes

`cargo test --workspace`: 90 (25 core + 8 cli + 1 ipc + 8 search + 48
vault). `cd ui && cargo test --lib`: 75. Total 165.
`cargo build --workspace` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

Validação manual contra `VaultAnotadinho`: todos os 6 subcomandos
testados na mão (`list-pages`, `--json`, `read`, `search`, `export
--folder pages`, `list-templates`, `new-from-template`), incluindo
caminho de erro (`read` de página inexistente → stderr + exit 1).

## Notas

`anotadinho-ipc` não depende de Tauri — o CLI só parseia argumentos e
chama os mesmos handlers que o app usa, sem lógica nova. Arquivos de
teste criados no vault durante a validação manual (`teste-cli-spec.md`)
foram removidos antes de fechar o ciclo; uma mudança não relacionada
que apareceu em `pages/exemplos-embeds.md` (drift do app Tauri rodando
em background nesta sessão) foi revertida também.

Próximo: corrigir o bug de achatamento de tabela ao salvar (111), e
depois placeholders de template além de `{{title}}` (112).
