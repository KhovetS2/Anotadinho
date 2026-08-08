---
id: "112"
titulo: "Placeholders de template alem de title"
status: pending
criado: 2026-08-08
autor: humano
prioridade: baixa
depende_de: ["100"]
estima_min: 45
agente_alvo: claude-sonnet
---

# Placeholders de template além de {{title}}

## Objetivo

`create_page_from_template` (ciclo 100) só substitui `{{title}}` —
gap notado na avaliação de prontidão pro agent-os: specs/decisões
datadas (o caso de uso mais citado pro tema) se beneficiam de
`{{date}}` já vindo preenchido, sem o usuário/agente precisar editar
manualmente logo depois de criar a página.

## Critérios de aceite

- [ ] `crates/vault/src/io.rs`: `create_page_from_template` ganha
      substituição de `{{date}}` (formato `YYYY-MM-DD`, mesmo padrão já
      usado em `open_today_journal`) além de `{{title}}`, aplicada no
      corpo E no frontmatter, na mesma passada
- [ ] Placeholders desconhecidos (`{{qualquercoisa}}`) NÃO são tocados —
      só os dois suportados (`{{title}}`, `{{date}}`); documentar isso
      no próprio arquivo de template de exemplo, pra não sugerir suporte
      que não existe
- [ ] `VaultAnotadinho/templates/` (exemplos criados no ciclo 100)
      atualizados pra usar `{{date}}` onde fizer sentido (ex: template
      de "Decisão" ganhando `date: {{date}}` no frontmatter)
- [ ] Teste novo em `crates/vault/src/io.rs` cobrindo substituição de
      `{{date}}` em corpo e frontmatter
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

- Motor de template genérico com variáveis arbitrárias definidas pelo
  usuário (ex: um formulário perguntando valores antes de criar) — os
  dois placeholders fixos (`{{title}}`, `{{date}}`) cobrem o caso comum;
  variáveis customizadas é um ciclo futuro bem maior se pedirem
- Formato de data customizável — sempre `YYYY-MM-DD`, mesmo padrão já
  usado em todo o resto do vault (`created`/`updated`/journals)

## Notas

Depende do ciclo 100 (`create_page_from_template` já existir). Mudança
pequena e isolada em `crates/vault/src/io.rs` — não deveria tocar UI
(o fluxo de "Nova página → escolher template → título" no `app.rs` já
passa o título; a data é calculada internamente na função, sem novo
input do usuário).
