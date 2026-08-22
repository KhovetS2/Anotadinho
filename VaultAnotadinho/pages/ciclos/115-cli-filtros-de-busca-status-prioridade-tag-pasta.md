---
title: Ciclo 115 — CLI filtros de busca status prioridade tag pasta
type: ciclo
ciclo: "115"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: ["110"]
tags:
- ciclo
---

# Ciclo 115 — CLI filtros de busca status prioridade tag pasta

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CLI filtros de busca (status/prioridade/tag/pasta)

## Objetivo

`list-pages` hoje só lista tudo; um agente que quer "specs em
`in-progress`" precisa ler cada página e parsear YAML na mão (gap
documentado no guia de agent-os, `pages/produto/guia-agent-os.md`).
Adiciona filtros opcionais direto no CLI.

## Critérios de aceite

- [x] `list-pages` ganha `--folder <prefix>` (filtra por prefixo de
      path, ex: `pages/specs`), `--tag <tag>` (uma ou mais, repetível),
      `--status <valor>` e `--priority <valor>` (lêem o frontmatter de
      cada página candidata)
- [x] Filtros combináveis (AND entre eles)
- [x] Sem nenhum filtro, comportamento idêntico ao de hoje
- [x] `--json` continua funcionando com os filtros aplicados
- [x] Testes de integração novos em `crates/cli/tests/cli.rs` cobrindo
      `--folder`, `--tag` e a combinação de `--status`+`--priority`
- [x] `cargo test --workspace` passa

## Comandos de validação

```bash
cargo test --workspace
cargo run -p anotadinho-cli -- --vault VaultAnotadinho list-pages --folder pages/specs
cargo run -p anotadinho-cli -- --vault VaultAnotadinho list-pages --tag spec --status backlog
```

## Não-objetivos

- Sintaxe de query complexa (OR, negação) — só AND entre filtros
  simples por enquanto
- Filtro por `depends_on`/campos de `extra` arbitrários — só os 4
  filtros nomeados acima; genérico fica pra outro ciclo se pedirem

## Notas

Implementado direto em `crates/cli` (não em `crates/vault`/`ipc`) —
é uma composição de `handle_list_pages` + `handle_read_page` +
`anotadinho_core::MarkdownCodec::split_frontmatter`, sem precisar de
uma operação nova na camada de vault.

## Resultado

# Ciclo 115 - done

## Resumo

`list-pages` do CLI ganha `--folder`/`--tag`/`--status`/`--priority`,
combináveis (AND). Fecha o gap documentado no guia de agent-os: antes,
um agente que queria "specs em `in-progress`" precisava ler cada
página e parsear YAML na mão.

## Arquivos criados/modificados

- `crates/cli/Cargo.toml` — nova dependência `anotadinho-core`
- `crates/cli/src/main.rs` — filtros em `ListPages`,
  `frontmatter_extra_str` helper
- `crates/cli/tests/cli.rs` — 4 testes novos, fixture de vault ganha
  uma página em `pages/specs/`

## Testes

`cargo test --workspace`: 101. `cd ui && cargo test --lib`: 75. Total 176.

Validado manualmente contra `VaultAnotadinho`: `--folder pages/specs`,
`--tag spec --status backlog` retornam exatamente a página esperada.

## Notas

Implementado direto no CLI (não em `crates/vault`/`ipc`) — composição
de handlers já existentes + `MarkdownCodec::split_frontmatter`, sem
operação nova na camada de vault.

Próximo: escrita de propriedades via CLI (116).
