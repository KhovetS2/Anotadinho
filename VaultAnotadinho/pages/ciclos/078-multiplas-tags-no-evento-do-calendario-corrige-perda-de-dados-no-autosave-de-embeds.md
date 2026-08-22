---
title: Ciclo 078 — Multiplas tags no evento do calendario + corrige perda de dados no autosave de embeds
type: ciclo
ciclo: "078"
status: concluida
date: 2026-08-07
prioridade: alta
depende_de: ["074"]
tags:
- ciclo
---

# Ciclo 078 — Multiplas tags no evento do calendario + corrige perda de dados no autosave de embeds

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Múltiplas tags no evento do calendário + corrige perda de dados no autosave de embeds

## Objetivo

`CalendarEntry` ganha múltiplas tags por evento (era uma só). Durante a
validação ao vivo, achei um bug sério e mais importante que a feature em
si: o autosave (debounced e o flush ao trocar de página, ambos do ciclo
074) podia persistir uma versão DESATUALIZADA do documento sempre que a
última ação antes de salvar fosse uma edição de EMBED (kanban/calendário/
tabela) — a edição em si acontecia normalmente na tela, mas o que ia pro
disco era a versão de ANTES daquela edição especificamente.

## Critérios de aceite

- [x] `CalendarEntry.tags: Vec<String>` — múltiplas tags, mesmo padrão já
      usado no kanban
- [x] Retrocompatível: campo antigo `tag: string` continua parseando
      (`legacy_tag`, só leitura) via `all_tags()`; primeira edição do
      evento migra pra `tags` de vez
- [x] `EventDetailModal`: clicar numa tag da paleta liga/desliga ela
      (multi-select), igual ao padrão das tags do card do kanban
- [x] **Bug crítico corrigido**: autosave (debounced + flush ao trocar de
      página) nunca mais persiste conteúdo desatualizado depois de uma
      edição de embed

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Cor por tag combinando múltiplas cores numa barra/bloco do calendário
  — continua usando só a PRIMEIRA tag pra colorir (barras são pequenas
  demais pra mostrar várias cores misturadas); todas as tags aparecem
  certas no modal

## Notas

**O bug crítico** (achado testando a feature de tags, não é sobre tags
em si): `mark_edited`/`trigger_debounced_save` (ciclo 074) recomputava o
markdown a partir de `content_md` DENTRO do próprio `spawn_local`/timer —
mas o `on_change` de um embed já tinha chamado `content_md.set(new_full)`
LOGO ANTES, no MESMO tick síncrono. Como um `UseStateHandle` capturado
numa closure fica congelado no valor de quando foi criado (`.set()`
chamado por outro clone do mesmo handle não é visto — mesma classe do bug
já achado no resize do calendário, ciclo 071), o recompute lia
`content_md` como estava ANTES do `.set()`, perdendo exatamente a edição
que acabou de disparar o autosave.

Confirmado ao vivo: abri o evento "Revisão de código" (tag legada
`urgente`), cliquei em "infra" pra adicionar uma segunda tag, e SEM
esperar nem interagir de novo, troquei de página imediatamente. O
arquivo no disco ficou só com `tags: [urgente]` — a tag "infra" que
acabei de clicar tinha sumido, mesmo a UI mostrando as duas como ativas
até o momento da troca.

**Fix**: extraído `persist(md: String)` (grava no disco + atualiza
estado "salvo", reaproveitado pelo botão "Salvar" manual e pelo
autosave) e `mark_edited(md: String)` (marca como editado + agenda o
timer de 3s) — os DOIS recebem o markdown JÁ CALCULADO como parâmetro em
vez de re-derivar de `content_md` depois. `on_edit` (texto puro) continua
recalculando do DOM ao vivo antes de chamar `mark_edited` (sempre correto
pra texto, já que a fonte de verdade ali é o DOM, não `content_md`).
Embeds e `insert_blank_line` (ciclo 075) chamam `mark_edited(new_full)`
direto com o valor que ELES mesmos acabaram de calcular, sem depender de
reler `content_md` nenhuma hora depois.

Reteste da mesma sequência (clicar "infra", trocar de página na hora)
confirmou `tags: [urgente, infra]` completo no disco depois do fix.

Lição de processo: o teste anterior que revelou o bug deixou o arquivo
`exemplos-embeds.md` num estado inconsistente no disco (a versão
desatualizada gravada pelo próprio bug), o que quebrou o teste
`exemplos_embeds_vault_file_parses` (fixture via `include_str!`) na
rodada seguinte — não era regressão nenhuma, só faltou reverter o
arquivo de teste antes de rodar `cargo test` de novo. `git checkout`
resolveu.

## Resultado

# Ciclo 078 - done

## Resumo

`CalendarEntry` ganha múltiplas tags (retrocompatível com o formato
antigo de uma tag só). Achado e corrigido durante a validação: bug
crítico no autosave (ciclo 074) que perdia a última edição de embed
sempre que ela fosse seguida de perto por um save (debounced ou flush ao
trocar de página).

## Arquivos criados/modificados

- `ui/src/embed.rs` — `CalendarEntry.tags: Vec<String>` +
  `legacy_tag: Option<String>` (retrocompat) + `all_tags()`, 2 testes
  novos
- `ui/src/components/embeds/event_detail_modal.rs` — UI de tags vira
  multi-select (toggle_tag)
- `ui/src/components/embeds/inline_calendar.rs` — 4 pontos usando
  `entry.tag` migrados pra `entry.all_tags().first()`
- `ui/src/components/editor.rs` — extraído `persist(md: String)` e
  `mark_edited(md: String)`, ambos recebendo o markdown já calculado em
  vez de re-derivar de `content_md` (fix do bug crítico)

## Testes

`cargo test --lib`: 54 passaram (2 novos:
`calendar_entry_legacy_single_tag_still_parses_via_all_tags`,
`calendar_entry_multiple_tags_roundtrip`).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Bug crítico confirmado e corrigido: abrir evento com tag legada
(`urgente`), clicar numa segunda tag ("infra") e trocar de página
IMEDIATAMENTE (sem esperar/interagir de novo) — antes do fix, o disco
ficava só com `tags: [urgente]` (a tag recém-clicada sumia); depois do
fix, `tags: [urgente, infra]` completo.

Multi-select confirmado: clicar duas tags diferentes deixa as DUAS
ativas na mesma vez (antes só uma podia estar ativa).

## Notas

O teste que revelou o bug deixou `exemplos-embeds.md` num estado
inconsistente no disco, quebrando `exemplos_embeds_vault_file_parses` na
rodada seguinte de `cargo test` — não era regressão, só faltou `git
checkout` no arquivo antes de rodar os testes de novo.
