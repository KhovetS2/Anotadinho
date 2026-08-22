---
title: Ciclo 096 — Gestao de assets
type: ciclo
ciclo: "096"
status: concluida
date: 2026-08-08
prioridade: baixa
depende_de: []
tags:
- ciclo
---

# Ciclo 096 — Gestao de assets

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Gestão de assets

## Objetivo

Décimo primeiro e último ciclo do conjunto grande. Página `type: assets`
listando arquivos em `assets/` com tamanho e se estão referenciados em
alguma página, com ação de excluir — fecha o conjunto de 11 melhorias
pedidas.

## Critérios de aceite

- [x] `crates/vault/src/io.rs`: `AssetInfo{path,size}`,
      `list_assets_info` (mantém `list_assets` original pro autocomplete
      do editor, que não precisa de tamanho), `delete_asset` (recusa
      qualquer path fora de `assets/`) — com 5 testes
- [x] `crates/ipc`/`src-tauri`: handlers + comandos Tauri
      `list_assets_info`/`delete_asset`
- [x] `ui/src/components/assets_page.rs` novo: tabela com tamanho
      formatado (B/KB/MB) e badge "usado"/"não usado" (1 scan de todas
      as páginas reaproveitado pra todos os assets, não 1 busca por
      asset), botão excluir com confirmação
- [x] Comando "Ver Assets" na paleta — cria/abre `pages/assets.md` se
      não existir
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

- Preview de imagem na lista — só nome + tamanho + status
- Renomear/mover asset a partir da página — só listar/excluir
- Limpeza em massa de todos os "não usados" de uma vez — exclui um por
  vez, com confirmação individual (evita apagar algo por engano)

## Notas

`delete_asset` reusa `resolve_safe` (mesma validação de path-traversal
de `read_page`/`delete_page`) mas adiciona uma checagem extra
(`starts_with("assets/")`) — sem isso a mesma função serviria (por
engano) pra excluir QUALQUER arquivo do vault, não só assets.

Detecção de "usado" é uma heurística simples (substring do path OU do
nome do arquivo no conteúdo bruto de todas as páginas) — não entende
sintaxe de markdown de verdade (um asset citado só em texto solto, não
como link/imagem, ainda contaria como "usado"). Aceitável pra v1: falso
positivo (marcar como usado algo que não é) é o lado seguro do erro,
já que o único efeito é não sugerir exclusão de algo que na real não
precisava ficar.

Validado ao vivo via MCP `tauri`: 2 arquivos de teste criados
diretamente em `assets/` (um citado numa página via `![img](assets/...)`,
outro não); página de Assets mostra "2 arquivos · 2.0 KB · 1 não
referenciados", badges corretos ("usado"/"não usado"); excluir o não
usado funciona (confirmado sumindo da lista E do disco). Arquivos de
teste e página `assets.md` criada removidos antes de fechar o ciclo.

Como os ciclos 090/096 tocaram `crates/*` (não só `ui/src/`), precisou
reiniciar o processo Tauri inteiro antes de testar ao vivo — mesma
lição já registrada no ciclo 094 (`trunk serve` recarrega o frontend
sozinho, mas o processo de backend só reinicia via
`./scripts/dev.sh` de novo).

## Resultado

# Ciclo 096 - done

## Resumo

Décimo primeiro e ÚLTIMO ciclo do conjunto grande pedido pelo usuário.
Página `type: assets` — arquivos em `assets/`, tamanho, uso, excluir.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `AssetInfo`, `list_assets_info`,
  `delete_asset`, 5 testes
- `crates/ipc/src/lib.rs` — handlers
- `src-tauri/src/main.rs` — comandos Tauri
- `ui/src/api.rs` — `AssetInfo`, `list_assets_info`, `delete_asset`
- `ui/src/components/assets_page.rs` — novo
- `ui/src/components/page_view.rs` — reconhece `"assets"`
- `ui/src/components/command_palette.rs`, `ui/src/app.rs` — comando
  "Ver Assets"
- `ui/src/components/mod.rs`, `ui/src/styles/main.css`

## Testes

`cargo test --workspace`: 61 (56 + 5 novos). `cd ui && cargo test
--lib`: 66. Total 127.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Lista mostra tamanho + status usado/não-usado corretos; excluir some da
lista e do disco. Detalhes no arquivo de task.

## Notas

**Último ciclo do conjunto de 11 itens pedidos pelo usuário nesta
sessão**: pastas, wikilinks, backlinks, landing page, calendário modo
Vault, paleta de comandos, vim mode, tags, busca full-text, undo/redo,
gestão de assets — todos entregues, testados e validados ao vivo.
