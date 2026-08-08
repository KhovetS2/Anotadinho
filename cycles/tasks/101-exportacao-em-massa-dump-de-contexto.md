---
id: "101"
titulo: "Exportacao em massa dump de contexto"
status: pending
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Exportação em massa / dump de contexto

## Objetivo

Quarto ciclo do tema "agent-os readiness". O `on_export` que já existe
(`editor.rs`) é só 1 página, a partir do HTML renderizado — bom pra
imprimir/compartilhar, ruim pra "colar isso tudo numa conversa com um
LLM". Este ciclo adiciona exportação de uma PASTA (ou seleção de
páginas) concatenando o MARKDOWN FONTE de cada uma num arquivo só, com
separadores.

## Critérios de aceite

- [ ] `crates/vault/src/io.rs`: `export_folder(folder_relative) ->
      Result<String>` — lê todas as páginas dentro da pasta (recursivo,
      reaproveita `list_pages` filtrado por prefixo de path), concatena
      o markdown de cada uma separado por `\n\n---\n\n## {título}\n\n`
      antes do conteúdo de cada página
- [ ] Botão/ação "Exportar pasta" na árvore de pastas da sidebar
      (ciclo 086) — baixa um `.md` só com o conteúdo concatenado
      (mesmo mecanismo de Blob+download já usado por `on_export`)
- [ ] Comando "Exportar vault inteiro" na paleta de comandos (mesma
      lógica, sem filtro de pasta — todas as páginas de `pages/` e
      `journals/`)
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
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
