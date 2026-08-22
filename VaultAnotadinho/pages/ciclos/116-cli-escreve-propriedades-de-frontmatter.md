---
title: Ciclo 116 — CLI escreve propriedades de frontmatter
type: ciclo
ciclo: "116"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: ["110"]
tags:
- ciclo
---

# Ciclo 116 — CLI escreve propriedades de frontmatter

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CLI escreve propriedades de frontmatter

## Objetivo

Fecha o loop do agent-os: hoje o CLI só cria página
(`new-from-template`); mudar `status`/`priority`/qualquer propriedade
depois exige abrir a GUI ou editar o `.md` na mão. Novo subcomando
`set-property` grava um campo de frontmatter preservando o corpo
intocado — mesmo princípio do painel de propriedades (ciclo 099).

## Critérios de aceite

- [x] `anotadinho-cli --vault <path> set-property <page_path> <key>
      <value>` — atualiza o frontmatter da página
- [x] Campos conhecidos (`title`, `type`) setam o campo tipado
      correspondente; `tags` aceita lista separada por vírgula; tudo
      mais vai pro `extra` (mesmo modelo de `Frontmatter`, ciclo 098)
- [x] Corpo da página (tudo depois do frontmatter) fica byte-a-byte
      intocado
- [x] Nova função pública `MarkdownCodec::set_frontmatter_field` em
      `crates/core`, coberta por 4 testes
- [x] Testes de integração no CLI cobrindo: setar um campo novo,
      sobrescrever um existente, e confirmar que o corpo não mudou
- [x] `cargo test --workspace` passa

## Comandos de validação

```bash
cargo test --workspace
cargo run -p anotadinho-cli -- --vault VaultAnotadinho set-property pages/specs/exemplo-exportar-nota-em-pdf.md status in-progress
cargo run -p anotadinho-cli -- --vault VaultAnotadinho read pages/specs/exemplo-exportar-nota-em-pdf.md
```

## Não-objetivos

- Validação de valores permitidos por campo (ex: recusar `status`
  fora do enum documentado) — v1 aceita qualquer string, igual o
  painel de propriedades na GUI
- Remover uma propriedade via CLI (`unset-property`) — só setar por
  enquanto; remoção fica pra outro ciclo se for pedida

## Notas

Extraído em `crates/core/src/markdown.rs` como
`MarkdownCodec::set_frontmatter_field` — mesmo princípio de
reconstrução que `on_frontmatter_change` em
`ui/src/components/editor.rs` (ciclo 099) já usa no frontend
(`split_frontmatter_text` pro corpo intocado + `serde_yaml::to_string`
pro bloco novo). `editor.rs` NÃO foi refatorado pra usar essa função
compartilhada nesta ciclo — o caminho dele já está testado e validado
ao vivo há vários ciclos; trocar por uma dependência nova sem
necessidade seria risco desnecessário. Fica disponível pra quando
fizer sentido consolidar.

Validado manualmente contra `VaultAnotadinho` real: `set-property` em
`status` preservou todos os outros campos (`date`/`priority`/`owner`/
`depends_on`/`related_decision`/`tags`) e o corpo inteiro; única
mudança cosmética é a ordem dos campos (mesmo trade-off já aceito
desde o ciclo 099 — `BTreeMap` em `extra` serializa em ordem
alfabética). Mudança de teste revertida do vault antes de fechar o
ciclo.

## Resultado

# Ciclo 116 - done

## Resumo

Novo subcomando `set-property` no CLI, fecha o loop do agent-os:
agora um agente cria página (`new-from-template`, ciclo 110) E
atualiza propriedades (`set-property`) sem precisar da GUI. Nova
função `MarkdownCodec::set_frontmatter_field` em `crates/core`,
reaproveitável por qualquer consumidor futuro.

## Arquivos criados/modificados

- `crates/core/src/markdown.rs` — `set_frontmatter_field`, 4 testes
- `crates/cli/src/main.rs` — subcomando `SetProperty`
- `crates/cli/tests/cli.rs` — 2 testes de integração

## Testes

`cargo test --workspace`: 101. `cd ui && cargo test --lib`: 75. Total 176.

Validado manualmente contra `VaultAnotadinho`: `set-property` em
`status` de uma spec real preservou todos os outros campos e o corpo
inteiro (mudança de teste revertida depois).

## Notas

`editor.rs` não foi refatorado pra usar a função compartilhada nesta
ciclo — caminho já testado/validado ao vivo, sem necessidade de
mexer. Fecha os 3 ciclos de melhoria do CLI levantados nesta rodada
(114/115/116).
