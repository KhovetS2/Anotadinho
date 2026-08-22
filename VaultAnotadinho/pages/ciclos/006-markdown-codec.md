---
title: "Ciclo 006 — MarkdownCodec: parse e serialize frontmatter + blocos"
type: ciclo
ciclo: "006"
status: concluida
date: 2026-08-04
prioridade: alta
depende_de: ["005"]
tags:
- ciclo
---

# Ciclo 006 — MarkdownCodec: parse e serialize frontmatter + blocos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# MarkdownCodec

## Objetivo

Implementar `MarkdownCodec::parse` e `serialize` em `crates/core`:
- Frontmatter YAML entre `---`
- Blocos como linhas `-` com indentação (depth)
- Properties inline `key:: value`
- Block ID via property `id:: uuid`

## Critérios de aceite

- [x] `parse` extrai frontmatter (title, tags)
- [x] `parse` cria blocos com content, depth, properties
- [x] `serialize` roundtrip básico (parse → serialize → parse)
- [x] Testes unitários cobrindo frontmatter, blocos, properties
- [x] Stubs antigos que esperavam erro são atualizados
- [x] `cargo test --workspace` exit 0
- [x] App continua compilando (sem regressão UI)

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```

## Não-objetivos

- Não integrar codec no editor ainda (ciclo futuro)
- Não headings especiais como BlockKind
- Não nested complex lists beyond depth

## Notas

Formato esperado:
```md
---
title: Foo
tags: [a]
---

- id:: uuid
  status:: done
  Conteudo do bloco
- outro bloco
```

## Resultado

## Resumo
Ciclo 006: MarkdownCodec parse/serialize.
- frontmatter YAML, blocos, depth, id::, properties
- roundtrip tests
