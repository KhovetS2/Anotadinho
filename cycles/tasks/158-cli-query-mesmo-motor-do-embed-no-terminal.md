---
id: "158"
titulo: "CLI query: mesmo motor do embed no terminal"
status: pending
criado: 2026-08-19
autor: humano
prioridade: media
depende_de: ["154"]
estima_min: 75
agente_alvo: claude-sonnet
---

# CLI query: mesmo motor do embed no terminal

## Objetivo

O ciclo 154 pôs o motor de consulta em `crates/core/src/query.rs`
justamente pra isso: expor no CLI o MESMO recorte que o embed mostra
na página. Sem isso o agente headless e o humano enxergam o vault por
critérios diferentes (o CLI tem `--status`/`--priority`/`--tag`
hardcoded do ciclo 115; o embed tem condições genéricas), e as duas
implementações divergem na primeira mudança.

## Critérios de aceite

- [ ] `anotadinho-cli query --from pages/specs --where status=backlog
      --where priority!=baixa --tag spec --sort priority --desc
      --limit 10 --json`
- [ ] `--where` repetível, aceitando `campo=valor` (eq), `campo!=valor`
      (neq), `campo~valor` (contains), `campo?` (exists),
      `campo>valor` / `campo<valor`
- [ ] Saída JSON com o MESMO schema do que o embed consome
      (`PageIndexEntry`), pra um agente conseguir correlacionar
- [ ] Saída legível (sem `--json`) em colunas: path, título e os
      campos citados nas condições/ordenação
- [ ] `--from-embed <page> <idx>`: roda a consulta declarada num embed
      `query` de uma página — o agente executa exatamente a view que o
      humano configurou na interface
- [ ] `list-pages --tag/--status/--priority/--folder` (ciclo 115)
      passa a delegar pro mesmo motor, sem duplicar filtro; o
      comportamento e os testes existentes do CLI continuam passando
- [ ] Testes em `crates/cli/tests/cli.rs` com vault temporário: cada
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

Atualizar a seção "Operando via CLI" do
`VaultAnotadinho/pages/produto/guia-agent-os.md` com os exemplos novos
faz parte deste ciclo — o guia é o contrato com o agente.
