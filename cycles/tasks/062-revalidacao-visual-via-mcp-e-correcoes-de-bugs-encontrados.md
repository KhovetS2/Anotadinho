---
id: "062"
titulo: "Revalidacao visual via MCP e correcoes de bugs encontrados"
status: done
criado: 2026-08-06
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
