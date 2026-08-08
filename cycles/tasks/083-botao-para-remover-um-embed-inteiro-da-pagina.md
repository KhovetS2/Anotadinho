---
id: "083"
titulo: "Botao para remover um embed inteiro da pagina"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: ["075"]
estima_min: 45
agente_alvo: claude-sonnet
---

# Botão para remover um embed inteiro da página

## Objetivo

Não existia nenhum jeito de tirar um kanban/calendário/tabela inteiro da
página depois de inserido — só dava pra apagar coisas DENTRO dele (uma
coluna, um card, uma linha). Adiciona um botão "✕" no canto do embed,
revelado no hover, que remove o segmento inteiro (com confirmação, já
que é destrutivo).

## Critérios de aceite

- [x] Botão "✕" no canto superior direito de cada embed, só visível no
      hover (mesmo padrão dos botões "+" de adicionar linha do ciclo 075)
- [x] Clicar pede confirmação (`PendingDialog::Confirm`, mesmo padrão já
      usado pra excluir coluna/card do kanban)
- [x] Confirmar remove só aquele segmento — os outros embeds/texto da
      página ficam intactos

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Desfazer/undo depois de remover — se remover por engano, only Ctrl+Z
  do sistema operacional (se o WebKitGTK suportar) ou reverter o arquivo
  manualmente

## Notas

Reaproveita a mesma estrutura de `insert_blank_line` (ciclo 075): recalcula
`segs` a partir de `content_md`, mexe no `Vec<DocSegment>` (dessa vez
`remove` em vez de `insert`), religa com `embed::join`, aplica via
`mark_edited` (ciclo 078 — evita o bug de staleness do autosave).

Validado ao vivo via MCP `tauri`: botão aparece no hover do Kanban Embed
de `exemplos-embeds.md`, clicar abre "Confirmar" com a mensagem certa,
confirmar remove só o kanban — "Calendar Embed" (heading + embed) logo
abaixo continua intacto.
