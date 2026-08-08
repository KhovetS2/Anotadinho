---
id: "111"
titulo: "Corrige achatamento de tabela ao salvar"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Corrige achatamento de tabela ao salvar

## Objetivo

Bug de perda de dado documentado desde o ciclo 099 e nunca corrigido:
salvar qualquer página com uma tabela Markdown (corpo solto, fora de
embed) achata a tabela pra texto corrido sem `|`, mesmo sem tocar em
nada relacionado a frontmatter — reproduzido com uma edição de corpo
comum + Salvar. Rastreado até o round-trip DOM→Markdown em
`recompute_markdown_from_dom`/`ui/src/html_to_md.rs`, que não trata
`<table>` corretamente ao reconverter a árvore do contenteditable de
volta pra markdown.

## Critérios de aceite

- [ ] `ui/src/html_to_md.rs`: caso `"table"` (ou equivalente) serializa
      `<table><thead>...<tbody>...` de volta pro formato
      `| a | b |\n|---|---|\n| ... |`, preservando cabeçalho e todas as
      linhas
- [ ] Teste novo em `ui/src/html_to_md.rs` (ou onde os testes de
      round-trip HTML→MD já vivem) cobrindo uma tabela simples de 2
      colunas x 2 linhas
- [ ] Reprodução manual documentada no ciclo 099
      (`cycles/tasks/099-*.md`, seção Notas) deixa de reproduzir:
      abrir uma página com tabela, editar o corpo (fora da tabela),
      salvar, e o `.md` no disco continua com `|` intactos
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

- Suporte a tabelas com alinhamento de coluna (`:---:`) ou células com
  markdown inline complexo — só precisa preservar o que já existia
  antes do round-trip quebrar (cabeçalho + linhas simples)
- Editar tabelas fora de embed pela UI WYSIWYG (adicionar linha/coluna
  clicando) — isso já existe pro embed `{{ type: "table" }}`; tabela
  markdown solta no corpo é só texto, edição é digitando markdown
  mesmo

## Notas

Página de teste conhecida que reproduz o bug (antes da correção):
`VaultAnotadinho/pages/sobre.md`, seção "## Stack" — tem uma tabela de
2 colunas. Reverter qualquer alteração de teste nesse arquivo do vault
antes de fechar o ciclo (`git checkout -- VaultAnotadinho/pages/sobre.md`
se precisar).
