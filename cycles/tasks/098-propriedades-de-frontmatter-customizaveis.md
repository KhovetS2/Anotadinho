---
id: "098"
titulo: "Propriedades de frontmatter customizaveis"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Propriedades de frontmatter customizáveis

## Objetivo

Primeiro ciclo do tema "agent-os readiness" (ver
`/home/elis/.claude/plans/agent-os-e-teclado.md`). `Frontmatter`
(`crates/core/src/page.rs`) é uma struct fixa (`title`/`tags`/
`created`/`updated`/`type`) sem catch-all — qualquer propriedade YAML
que não seja uma dessas é silenciosamente DESCARTADA ao salvar. Isso
bloqueia templates de verdade e um painel de propriedades (ciclos
099/100): sem preservar propriedades arbitrárias, um agente-os que
queira usar `status:`/`owner:`/`spec-id:`/etc no frontmatter de uma
página vê esses campos desaparecerem no primeiro save feito pela UI.

## Critérios de aceite

- [ ] `Frontmatter` ganha `#[serde(flatten)] extra: BTreeMap<String,
      serde_yaml::Value>` — qualquer chave YAML não reconhecida
      (diferente de title/tags/created/updated/type) é preservada
- [ ] `serialize()` (`crates/core/src/markdown.rs`) emite `extra` de
      volta no YAML, em ordem estável (`BTreeMap` já ordena por chave)
- [ ] Round-trip: parse → serialize → parse de novo produz os MESMOS
      valores em `extra` (teste novo)
- [ ] Retrocompatibilidade: frontmatter só com os campos fixos de
      sempre continua parseando/serializando IDÊNTICO a antes (nenhum
      teste existente quebra)
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

- UI pra editar `extra` — isso é o ciclo 099 (painel de propriedades),
  este ciclo é só a camada de dados (`crates/core`)
- Validação de schema (ex: "toda página `type: spec` precisa ter
  `status:`") — fica pra se algum dia for pedido, fora de escopo aqui
- Tipos ricos pra valores de `extra` (datas, números, booleanos com
  parsing especial) — guarda como `serde_yaml::Value` genérico, a UI
  do ciclo 099 decide como exibir/editar cada tipo

## Notas

`#[serde(flatten)]` com `BTreeMap<String, serde_yaml::Value>` é o
padrão idiomático do serde pra "capturar o resto" — `serde_yaml`
suporta isso nativamente. Único cuidado: `flatten` interage mal com
alguns combos de `Option`/`default` em structs complexas — testar
especificamente que `tags: Vec<String>` (que já usa
`#[serde(default)]`) continua funcionando junto com o novo `extra`
flatten, escrevendo um teste de round-trip com AMBOS presentes (tags E
propriedades customizadas na mesma página) antes de considerar o ciclo
fechado.
