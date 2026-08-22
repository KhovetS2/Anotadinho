---
title: "Ciclo 158 — CLI query: mesmo motor do embed no terminal"
type: ciclo
ciclo: "158"
status: concluida
date: 2026-08-19
prioridade: media
depende_de: ["154"]
tags:
- ciclo
---

# Ciclo 158 — CLI query: mesmo motor do embed no terminal

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CLI query: mesmo motor do embed no terminal

## Objetivo

O ciclo 154 pôs o motor de consulta em `crates/core/src/query.rs`
justamente pra isso: expor no CLI o MESMO recorte que o embed mostra
na página. Sem isso o agente headless e o humano enxergam o vault por
critérios diferentes (o CLI tem `--status`/`--priority`/`--tag`
hardcoded do ciclo 115; o embed tem condições genéricas), e as duas
implementações divergem na primeira mudança.

## Critérios de aceite

- [x] `anotadinho-cli query --from pages/specs --where status=backlog
      --where priority!=baixa --tag spec --sort priority --desc
      --limit 10 --json`
- [x] `--where` repetível, aceitando `campo=valor` (eq), `campo!=valor`
      (neq), `campo~valor` (contains), `campo?` (exists),
      `campo>valor` / `campo<valor`
- [x] Saída JSON com o MESMO schema do que o embed consome
      (`PageIndexEntry`), pra um agente conseguir correlacionar
- [x] Saída legível (sem `--json`) em colunas: path, título e os
      campos citados nas condições/ordenação
- [x] `--from-embed <page>:<idx>` (dois-pontos, não espaço — clap não
      aceita um argumento longo com dois valores): roda a consulta
      declarada num embed `query` de uma página
- [x] `list-pages --tag/--status/--priority/--folder` (ciclo 115)
      passa a delegar pro mesmo motor, sem duplicar filtro; o
      comportamento e os testes existentes do CLI continuam passando
- [x] Testes em `crates/cli/tests/cli.rs` com vault temporário: cada
      operador, ordenação, limite, `--from-embed` e paridade entre
      `list-pages --status X` e `query --where status=X`

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo test -p anotadinho-cli
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Saída em CSV/tabela markdown
- OR/parênteses (mesma decisão do ciclo 154)
- Substituir `search` (FTS5) — busca full-text é outra coisa; `query`
  filtra metadado

## Notas

`cargo test -p anotadinho-cli`: 32 (22 + 10 novos). `cargo test
--workspace`: 251. `trunk build` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

`list-pages` delegou o filtro pro motor, mas continua imprimindo o NOME
DO ARQUIVO como título (e não o `title` do frontmatter, que é o que
`PageIndexEntry` prefere): a saída dele é contrato de script de agente,
e o teste `list_pages_json_emits_valid_json` pegou a mudança na hora.
`query` usa o título do frontmatter, que é o certo pra consulta — a
diferença está comentada no código.

Efeito colateral bom da delegação: `--status`/`--priority` do
`list-pages` agora enxergam também property de corpo (`status:: x`),
não só frontmatter YAML.

Atualizar a seção "Operando via CLI" do
`VaultAnotadinho/pages/produto/guia-agent-os.md` com os exemplos novos
faz parte deste ciclo — o guia é o contrato com o agente.

## Resultado

# Ciclo 158 - done

## Resumo

`anotadinho-cli query` expõe no terminal o mesmo motor do embed
`{{ type: "query" }}` (ciclo 154). Com `--from-embed
<page>:<idx>` o agente executa literalmente a view que o humano
configurou na página.

`list-pages --tag/--status/--priority/--folder` (ciclo 115) passou a
delegar pro mesmo motor, em vez de manter o filtro próprio que lia
página por página — duas implementações do mesmo conceito divergiriam
na primeira mudança.

## Arquivos criados/modificados

- `crates/cli/src/main.rs` — subcomando `query`, `parse_condition`,
  `query_from_embed`, `list-pages` delegando pro motor
- `crates/cli/tests/cli.rs` — 10 testes novos
- `VaultAnotadinho/pages/produto/guia-agent-os.md` — seções novas de
  consulta e de embeds (o guia é o contrato com o agente)

## Testes adicionados

- filtro por pasta + condição; tags em AND; ordenação com limite
- `!=` inclui página sem o campo; `?`, `~` e `>` numérico
- condição malformada falha com mensagem útil
- `--from-embed` roda a consulta da página; em embed de outro tipo falha
- `--json` usa o schema de `PageIndexEntry`
- **paridade**: `list-pages --status X` e `query --where status=X`
  devolvem exatamente os mesmos paths

## Problemas encontrados

- A delegação trocou o título impresso por `list-pages` (nome do
  arquivo → `title` do frontmatter). Revertido: a saída é contrato de
  script. O teste existente pegou na hora.
- `--from-embed` usa `page:índice` num argumento só — clap não aceita
  dois valores num `--arg` longo.

## Notas para próximos ciclos

- Leitura e escrita do agente estão fechadas (157 + 158).
- Restam 159 (mover/duplicar embed) e 160 (painel), mais as tasks de
  bug 161/162 e a 163.
