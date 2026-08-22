---
title: Ciclo 062 — Revalidacao visual via MCP e correcoes de bugs encontrados
type: ciclo
ciclo: "062"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 062 — Revalidacao visual via MCP e correcoes de bugs encontrados

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Revalidação visual via MCP e correções de bugs encontrados

## Objetivo

Completar a validação pendente do ciclo 060 (renderização real de
`exemplos-embeds.md` no app rodando) usando o MCP `tauri` (screenshot +
DOM real, não só testes unitários), e corrigir os bugs reais que essa
validação expôs — que os testes unitários não cobriam.

## Critérios de aceite

- [x] MCP bridge (`tauri-plugin-mcp-bridge`) funcional: faltava
      `"withGlobalTauri": true` em `tauri.conf.json` (o plugin já estava
      registrado em `src-tauri/src/main.rs`, isso não precisou mudar)
- [x] `exemplos-embeds.md` confirmado renderizando kanban/calendar/table
      como componentes reais no app rodando (screenshot), não só nos testes
- [x] Bug encontrado: `.kanban`/`.calendar__*`/`.task-table__*` nunca
      tiveram CSS escrito (nem os componentes whole-page antigos, nem os
      embeds novos) — tudo renderizava como texto corrido. CSS adicionado.
- [x] Bug encontrado: `Frontmatter.created`/`updated` como
      `Option<DateTime<Utc>>` falhava em parsear `created: YYYY-MM-DD`
      (formato usado em várias páginas reais do vault) — o erro de parse
      acionava o fallback pro texto bruto, reproduzindo o bug original do
      frontmatter em qualquer página com esse campo. Trocado pra
      `Option<String>` (livre, sem parsing).
- [x] Bug encontrado (mais sério): salvar uma página com embeds via
      `do_save` podia grudar um heading editado na fence seguinte sem quebra
      de linha (`## Kanban Embed` + ` ```kanban ` na mesma linha),
      corrompendo o arquivo — `html_to_markdown` não preserva o `\n` final,
      e `embed::join()` concatenava direto. Testado ao vivo: editei uma
      célula da tabela no app rodando, salvei, e vi a corrupção no arquivo
      antes do fix. `join()` agora garante a quebra de linha.
- [x] Editar um embed (testado: célula de tabela) e salvar persiste
      corretamente no arquivo, confirmado lendo o arquivo do disco após o
      save e recarregando a página no app

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Não fiz "preservar formatação original de embeds não editados" — hoje
  todo save reconstrói TODOS os embeds da página a partir do `EmbedData`
  parseado (normaliza case de coluna, largura do separador de tabela,
  etc), mesmo os que não foram tocados. Não é corrupção (dado preservado),
  só reformatação cosmética. Documentado, não corrigido — resolver isso
  exigiria rastrear quais segmentos foram de fato editados.
- Não gatei `.plugin(tauri_plugin_mcp_bridge::init())` atrás de
  `#[cfg(debug_assertions)]` em `main.rs` (a doc do próprio plugin
  recomenda isso pra não ir pra build de release) — já estava assim antes,
  não fazia parte do que foi pedido/aprovado nesta rodada.

## Notas

Sem essa validação ao vivo (só com os testes unitários dos ciclos 060/061),
os 3 bugs acima teriam ido pro ar sem ninguém notar — nenhum deles quebrava
`cargo test --workspace`. Fica como argumento a favor de manter o hábito de
`ui-check` antes de considerar um ciclo de UI realmente fechado.

## Resultado

# Ciclo 062 - done

## Resumo

Fecha a validação que tinha ficado pendente no ciclo 060 (rodar o app de
verdade via MCP `tauri`, não só testes unitários). Isso exigiu completar a
config do MCP bridge (`withGlobalTauri: true`, com aprovação explícita do
usuário) e expôs 3 bugs reais que nenhum teste unitário pegava: CSS
inexistente pra kanban/calendar/task-table (whole-page e embed inline),
`Frontmatter.created` quebrando com datas sem horário (formato usado em
várias páginas reais do vault), e uma corrupção de arquivo ao salvar uma
página com embeds (heading colava na fence seguinte por falta de `\n`).

## Arquivos criados/modificados

- `src-tauri/tauri.conf.json` — `withGlobalTauri: true`
- `ui/src/styles/main.css` — CSS novo pra `.kanban*`/`.calendar*`/`.task-table*`
- `crates/core/src/page.rs` — `Frontmatter.created`/`updated`: `DateTime<Utc>` → `String`
- `crates/core/Cargo.toml` — remove dependência `chrono` (ficou sem uso)
- `ui/src/embed.rs` — `join()` garante `\n` entre um segmento de markdown editado e o próximo

## Testes adicionados

- `crates/core`: `parse_frontmatter_bare_date_created` (regressão do bug do `created:`)
- `ui`: `join_inserts_missing_newline_before_next_segment`, `join_does_not_duplicate_existing_newline`

## Problemas encontrados

Todos os 3 listados no Resumo — detalhados em `cycles/tasks/062-*.md`. Todos corrigidos
e revalidados ao vivo no app rodando (screenshot antes/depois, e teste de edição real
de uma célula da tabela + save + reload confirmando que não corrompe mais).

## Notas para próximos ciclos

- O MCP bridge (`tauri-plugin-mcp-bridge`) agora funciona nesta máquina — `driver_session
  action: start` conecta em `localhost:9223` e os tools `webview_screenshot`/
  `webview_execute_js`/`webview_interact`/`webview_dom_snapshot` funcionam de verdade.
  Vale usar o skill `ui-check` (ou chamar os tools direto) antes de fechar qualquer
  ciclo que mexa em UI — os 3 bugs deste ciclo não apareciam em `cargo test`.
- Reformatação cosmética de embeds não editados a cada save é um comportamento
  conhecido, não corrigido (ver "Não-objetivos" da task 062).
