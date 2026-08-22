---
title: Ciclo 101 — Exportacao em massa dump de contexto
type: ciclo
ciclo: "101"
status: concluida
date: 2026-08-08
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 101 — Exportacao em massa dump de contexto

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Exportação em massa / dump de contexto

## Objetivo

Quarto ciclo do tema "agent-os readiness". O `on_export` que já existe
(`editor.rs`) é só 1 página, a partir do HTML renderizado — bom pra
imprimir/compartilhar, ruim pra "colar isso tudo numa conversa com um
LLM". Este ciclo adiciona exportação de uma PASTA (ou seleção de
páginas) concatenando o MARKDOWN FONTE de cada uma num arquivo só, com
separadores.

## Critérios de aceite

- [x] `crates/vault/src/io.rs`: `export_folder(folder_relative) ->
      Result<String>` — lê todas as páginas dentro da pasta (recursivo,
      reaproveita `list_pages` filtrado por prefixo de path), concatena
      o markdown de cada uma separado por `\n\n---\n\n## {título}\n\n`
      antes do conteúdo de cada página
- [x] Botão/ação "Exportar pasta" na árvore de pastas da sidebar
      (ciclo 086) — baixa um `.md` só com o conteúdo concatenado
      (mesmo mecanismo de Blob+download já usado por `on_export`)
- [x] Comando "Exportar vault inteiro" na paleta de comandos (mesma
      lógica, sem filtro de pasta — todas as páginas de `pages/` e
      `journals/`)
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

- Seleção manual de páginas específicas (checkbox por página) — v1 é
  só "pasta inteira" ou "vault inteiro"; seleção granular é ciclo
  futuro se o "pasta inteira" não for suficiente na prática
- Formatos além de markdown puro concatenado (PDF, HTML) — o `on_export`
  de 1 página já cobre "bonito pra ler"; este aqui é deliberadamente
  cru, pensado pra contexto de LLM
- Excluir frontmatter do dump — mantém o frontmatter de cada página
  visível no dump (pode ser informação relevante pro agente que for ler)

## Notas

Diferente do `on_export` existente (lê o DOM renderizado — só funciona
pra 1 página aberta na hora), este usa `read_page` direto do disco pra
cada página da pasta — mais simples e mais correto pra esse caso de uso
(não depende de nenhuma página estar aberta/renderizada).

Um único comando IPC `export_folder` cobre os dois casos: `folder_path`
vazio exporta o vault inteiro (`export_vault` em `VaultIo` é só um
atalho pra isso), evitando duplicar handler/comando/wrapper.

Novo módulo `ui/src/download.rs`: `download_text_file(filename, mime,
content)` — Blob + `<a download>` sintético, reusado pelo botão da
sidebar e pelo comando da paleta. Precisou adicionar
`BlobPropertyBag`/`HtmlAnchorElement` às features do `web-sys` em
`ui/Cargo.toml`.

Validado ao vivo via MCP `tauri`: comando direto (`invoke`) confirma
o dump do vault inteiro (11153 chars, cabeçalhos `## título`
corretos); botão "Exportar pasta" na árvore da sidebar baixou
`Nova_pasta.md` (pasta de teste vazia → conteúdo vazio, como esperado);
comando "Exportar vault inteiro" da paleta baixou `vault.md` (12KB,
524 linhas) em `~/Downloads/` — sem diálogo nativo bloqueante, o
`<a download>` funciona direto no webview do Tauri.

## Resultado

# Ciclo 101 - done

## Resumo

Quarto ciclo do tema "agent-os readiness". Exportação em massa: dump
do markdown fonte (frontmatter incluído) de uma pasta inteira ou do
vault todo, concatenado num `.md` só com cabeçalhos por página —
pensado pra colar num contexto de LLM, diferente do `on_export` de 1
página (HTML renderizado) que já existia.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `export_folder`, `export_vault`, 3 testes
  novos
- `crates/ipc/src/lib.rs` — `handle_export_folder`
- `src-tauri/src/main.rs` — comando `export_folder` registrado
- `ui/src/api.rs` — wrapper `export_folder`
- `ui/src/download.rs` (novo) — `download_text_file` (Blob + `<a
  download>`), reusado pelos dois pontos de entrada
- `ui/Cargo.toml` — features `BlobPropertyBag`/`HtmlAnchorElement` do
  web-sys
- `ui/src/components/sidebar.rs` — botão "Exportar pasta" em cada nó da
  árvore de pastas
- `ui/src/components/command_palette.rs` — comando "Exportar vault
  inteiro"
- `ui/src/app.rs` — `export_vault_action`

## Testes

`cargo test --workspace`: 74 (25 core + 1 ipc + 8 search + 40 vault).
`cd ui && cargo test --lib`: 66. Total 140.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: dump do vault inteiro via `invoke`
direto confere (11153 chars, cabeçalhos corretos); botão da sidebar e
comando da paleta testados de ponta a ponta, arquivo baixado em
`~/Downloads/` com o conteúdo esperado.

## Notas

Detalhes de implementação no arquivo de task (comando IPC único
cobrindo pasta/vault, novo módulo de download reusável).

Próximo: busca full-text real na paleta de comandos (102) — hoje a
paleta só busca por título de página, não por conteúdo.
