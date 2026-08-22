---
title: Ciclo 112 — Placeholders de template alem de title
type: ciclo
ciclo: "112"
status: concluida
date: 2026-08-08
prioridade: baixa
depende_de: ["100"]
tags:
- ciclo
---

# Ciclo 112 — Placeholders de template alem de title

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Placeholders de template além de {{title}}

## Objetivo

`create_page_from_template` (ciclo 100) só substitui `{{title}}` —
gap notado na avaliação de prontidão pro agent-os: specs/decisões
datadas (o caso de uso mais citado pro tema) se beneficiam de
`{{date}}` já vindo preenchido, sem o usuário/agente precisar editar
manualmente logo depois de criar a página.

## Critérios de aceite

- [x] `crates/vault/src/io.rs`: `create_page_from_template` ganha
      substituição de `{{date}}` (formato `YYYY-MM-DD`, mesmo padrão já
      usado em `open_today_journal`) além de `{{title}}`, aplicada no
      corpo E no frontmatter, na mesma passada
- [x] Placeholders desconhecidos (`{{qualquercoisa}}`) NÃO são tocados —
      só os dois suportados (`{{title}}`, `{{date}}`), coberto por
      teste dedicado
- [x] `VaultAnotadinho/templates/` atualizados: "Decisão" ganha
      `date: {{date}}` no frontmatter + linha de corpo; "Nota de
      reunião" ganha `date: {{date}}` no frontmatter (spec.md não
      precisa de data, mantido como estava)
- [x] Teste novo em `crates/vault/src/io.rs` cobrindo substituição de
      `{{date}}` em corpo e frontmatter, e teste separado confirmando
      que placeholder desconhecido não é tocado
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

- Motor de template genérico com variáveis arbitrárias definidas pelo
  usuário (ex: um formulário perguntando valores antes de criar) — os
  dois placeholders fixos (`{{title}}`, `{{date}}`) cobrem o caso comum;
  variáveis customizadas é um ciclo futuro bem maior se pedirem
- Formato de data customizável — sempre `YYYY-MM-DD`, mesmo padrão já
  usado em todo o resto do vault (`created`/`updated`/journals)

## Notas

Depende do ciclo 100 (`create_page_from_template` já existir). Mudança
pequena e isolada em `crates/vault/src/io.rs` — não tocou UI (o fluxo
de "Nova página → escolher template → título" no `app.rs` já passa o
título; a data é calculada internamente na função, sem novo input do
usuário).

Validado via CLI (ciclo 110): `anotadinho-cli new-from-template
templates/decisao.md "Teste"` produziu `date: 2026-08-08` no
frontmatter e `_Decisão registrada em 2026-08-08._` no corpo. Arquivo
de teste removido do vault depois.

## Resultado

# Ciclo 112 - done

## Resumo

`create_page_from_template` (ciclo 100) ganha substituição de
`{{date}}` (formato `YYYY-MM-DD`) além de `{{title}}`, aplicada em
corpo e frontmatter na mesma passada. Placeholders desconhecidos
seguem intocados. Templates de exemplo do vault ("Decisão", "Nota de
reunião") atualizados pra usar `{{date}}`.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `create_page_from_template` ganha
  `.replace("{{date}}", &today)`, 2 testes novos
- `VaultAnotadinho/templates/decisao.md` — `date: {{date}}` +
  linha de corpo
- `VaultAnotadinho/templates/nota-de-reuniao.md` — `date: {{date}}`

## Testes

`cargo test --workspace`: 92. `cd ui && cargo test --lib`: 75. Total 167.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validado via `anotadinho-cli new-from-template` (ciclo 110) contra o
template de Decisão — `{{date}}` resolvido corretamente em frontmatter
e corpo; arquivo de teste removido do vault depois.

## Notas

Fecha os 3 ciclos levantados na avaliação de prontidão pro agent-os
(110 CLI headless, 111 bug de tabela, 112 placeholder de data).
