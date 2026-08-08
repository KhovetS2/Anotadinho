---
id: "098"
titulo: "Propriedades de frontmatter customizaveis"
status: done
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
(`crates/core/src/page.rs`) era uma struct fixa (`title`/`tags`/
`created`/`updated`/`type`) sem catch-all — qualquer propriedade YAML
que não fosse uma dessas era descartada quando lida por
`serde_yaml::from_str` NESSA struct. Necessário como base de dados pro
painel de propriedades (099) e templates (100) conseguirem ler/escrever
QUALQUER propriedade de frontmatter, não só as 5 fixas.

## Critérios de aceite

- [x] `Frontmatter` ganha `#[serde(flatten)] extra: BTreeMap<String,
      serde_yaml::Value>` — qualquer chave YAML não reconhecida
      (diferente de title/tags/created/updated/type) é preservada
- [x] `serialize()` (`crates/core/src/markdown.rs`) emite `extra` de
      volta no YAML, em ordem estável (`BTreeMap` já ordena por chave)
- [x] Round-trip: parse → serialize → parse de novo produz os MESMOS
      valores em `extra` (3 testes novos em `page.rs` + 1 em
      `markdown.rs`)
- [x] Retrocompatibilidade: frontmatter só com os campos fixos de
      sempre continua parseando/serializando IDÊNTICO a antes (todos os
      testes existentes continuam passando)
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

- UI pra editar `extra` — isso é o ciclo 099 (painel de propriedades),
  este ciclo é só a camada de dados (`crates/core`)
- Validação de schema (ex: "toda página `type: spec` precisa ter
  `status:`") — fica pra se algum dia for pedido, fora de escopo aqui
- Tipos ricos pra valores de `extra` (datas, números, booleanos com
  parsing especial) — guarda como `serde_yaml::Value` genérico, a UI
  do ciclo 099 decide como exibir/editar cada tipo

## Notas

**Correção importante ao objetivo original**: investigando antes de
implementar, descobri que `MarkdownCodec::serialize` (o caminho TIPADO,
lossy, que descartaria propriedades desconhecidas) **nunca é chamado
pela UI** — `grep -rn "MarkdownCodec::serialize" --include="*.rs"` só
acha o próprio crate `core` e seus testes. O fluxo real de
salvar/editar do editor usa `split_frontmatter_text` (preserva o texto
CRU do frontmatter, verbatim, byte a byte) — ou seja, propriedades
customizadas escritas manualmente no arquivo `.md` **já sobreviviam**
a edições feitas pela UI antes deste ciclo, elas só nunca apareciam
visíveis/editáveis em lugar nenhum da interface (não existe NENHUMA UI
hoje pra ver/editar frontmatter, nem os campos fixos). O gap real não
era "perda de dado", era "falta de um modelo de dados genérico pra
alimentar uma UI de propriedades" — que é exatamente o que `extra`
resolve, preparando o terreno pro ciclo 099 sem mudar a premissa: o
`Frontmatter` tipado (com `extra` agora) vira a ÚNICA fonte usada pela
NOVA UI de propriedades pra ler E escrever de volta (diferente do resto
do editor, que continua com preservação de texto cru pra tudo que não
passa pelo painel).

`#[serde(flatten)]` com `BTreeMap<String, serde_yaml::Value>` funcionou
sem atrito nenhum com os campos `Option`/`#[serde(default)]` já
existentes (`tags`, `created`, etc.) — testado explicitamente com tags
E propriedades customizadas na mesma página.
