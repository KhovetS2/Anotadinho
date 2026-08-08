---
id: "116"
titulo: "CLI escreve propriedades de frontmatter"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: ["110"]
estima_min: 60
agente_alvo: claude-sonnet
---

# CLI escreve propriedades de frontmatter

## Objetivo

Fecha o loop do agent-os: hoje o CLI só cria página
(`new-from-template`); mudar `status`/`priority`/qualquer propriedade
depois exige abrir a GUI ou editar o `.md` na mão. Novo subcomando
`set-property` grava um campo de frontmatter preservando o corpo
intocado — mesmo princípio do painel de propriedades (ciclo 099).

## Critérios de aceite

- [ ] `anotadinho-cli --vault <path> set-property <page_path> <key>
      <value>` — atualiza o frontmatter da página
- [ ] Campos conhecidos (`title`, `type`) setam o campo tipado
      correspondente; `tags` aceita lista separada por vírgula; tudo
      mais vai pro `extra` (mesmo modelo de `Frontmatter`, ciclo 098)
- [ ] Corpo da página (tudo depois do frontmatter) fica byte-a-byte
      intocado
- [ ] Nova função pública em `crates/vault` (ou reaproveita
      `anotadinho_core::MarkdownCodec`) que faz esse
      parse→atualiza→serializa, coberta por teste
- [ ] Testes de integração no CLI cobrindo: setar um campo novo,
      sobrescrever um existente, e confirmar que o corpo não mudou
- [ ] `cargo test --workspace` passa

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

Mesma lógica de reconstrução que `on_frontmatter_change` em
`ui/src/components/editor.rs` (ciclo 099) já usa no frontend — vale
extrair essa lógica (parse frontmatter → mutação → serializa +
recombina com o corpo original) pra uma função compartilhada em
`crates/vault` ou `crates/core`, já que agora tem DOIS consumidores
(editor e CLI) fazendo a mesma coisa.
