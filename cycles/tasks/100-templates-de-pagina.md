---
id: "100"
titulo: "Templates de pagina"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: ["098"]
estima_min: 120
agente_alvo: claude-sonnet
---

# Templates de página

## Objetivo

Terceiro ciclo do tema "agent-os readiness" — o item mais pedido pra
esse uso (specs/decisões/registros seguindo uma estrutura repetida).
Templates são páginas normais guardadas em `templates/` na raiz do
vault (pasta visível, não oculta — o usuário edita/cria os próprios
templates como qualquer outra página); criar página a partir de um
template copia o corpo + frontmatter (incluindo `extra`, ciclo 098),
substituindo `{{title}}` pelo título escolhido.

## Critérios de aceite

- [ ] `crates/vault/src/io.rs`: `create_page_from_template(template_path,
      title, folder) -> Result<PageMeta>` — lê o template, substitui
      `{{title}}` (corpo E frontmatter) pelo título digitado, escreve a
      página nova (mesma lógica de slug único de `create_page_in`)
- [ ] `list_templates() -> Result<Vec<PageMeta>>` — lista arquivos em
      `templates/` (mesmo padrão de `list_pages`, mas escaneando só essa
      pasta)
- [ ] Fluxo de "Nova página" (Ctrl+N/paleta) ganha uma etapa opcional de
      escolher um template (lista vazia = comportamento atual,
      inalterado, se `templates/` não existir ou estiver vazia)
- [ ] `VaultAnotadinho/templates/` ganha 2-3 exemplos reais (ex: "Spec",
      "Decisão", "Nota de reunião") documentando o recurso pro próprio
      vault de demonstração
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

- Placeholders além de `{{title}}` (ex: `{{date}}`, variáveis
  customizadas) — v1 é só o título, que já cobre o caso mais comum;
  expandir os placeholders é um ciclo futuro se pedirem
- Templates por tipo de página com lógica especial (kanban/calendar já
  têm seu próprio jeito de nascer preenchidos) — templates aqui são só
  pra páginas markdown normais (`type: md` ou sem type)
- Editor visual de template — é um arquivo `.md` normal, editado como
  qualquer página

## Notas

Depende do ciclo 098 (`Frontmatter.extra`) pra templates poderem trazer
propriedades customizadas (ex: um template de "Spec" já vindo com
`status: draft`) sem perdê-las ao copiar.

`templates/` fica FORA de `pages/`/`journals/` — não deveria aparecer
na lista normal de páginas da sidebar nem ser contada como "página" em
lugar nenhum (tags, busca, backlinks); confirmar que `list_pages()`
(que só varre `pages/`/`journals/`) já ignora `templates/` por
construção (deveria, já que só escaneia essas duas pastas fixas).
