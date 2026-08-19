---
id: "151"
titulo: "Embed callout: caixa de destaque colapsável"
status: pending
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: ["148"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Embed callout: caixa de destaque colapsável

## Objetivo

Primeiro embed de composição visual (não de banco de dados). Hoje pra
destacar um aviso numa nota só existe `>` (blockquote), que tem um
estilo só e não colapsa. O callout é o bloco mais usado de
Notion/Obsidian pra montar nota personalizada: caixa colorida por
variante, com título, ícone e corpo markdown de verdade dentro.

Este ciclo também extrai o helper `embed_markdown_field` — um campo de
markdown editável DENTRO de um embed — que o ciclo 152 (columns)
reusa.

## Critérios de aceite

- [ ] `EmbedKind::Callout` + `{{ type: "callout" }}` reconhecido por
      `segment`
- [ ] `CalloutEmbedData { variant: CalloutVariant (Info|Success|
      Warning|Error|Tip), title: String, collapsed: bool, body: String }`
      com parse/serialize por derive de serde (nunca montando string)
- [ ] Componente `embeds/inline_callout.rs`: header com ícone da
      variante + título editável inline + botão de colapsar; corpo
      renderizado por `markdown_render::render`
- [ ] `ui/src/components/embeds/markdown_field.rs` (novo):
      componente `EmbedMarkdownField` — recebe markdown, injeta o HTML
      renderizado num `contenteditable` próprio, e no `oninput` (com
      debounce) converte de volta por `html_to_md::html_to_markdown` e
      emite `Callback<String>`. NÃO participa de `segment_refs`: a
      mutação viaja pelo `on_change` do embed, igual kanban/tabela
- [ ] Trocar a variante por um seletor no header (sem modal)
- [ ] Ícones novos em `icon.rs`: `info`, `alert-triangle`,
      `alert-circle`, `lightbulb` (o `check` já existe pra Success)
- [ ] CSS BEM `.callout` / `.callout--{variant}` só com tokens; fundo
      via `color-mix(in srgb, var(--accent-...) 12%, transparent)`
- [ ] `data-nav-item`/`data-nav-group` no header e nos controles, foco
      visível
- [ ] Testes de round-trip: parse(serialize(x)) == x, incluindo corpo
      multi-linha com `:` e `#` (o caso que quebrou no ciclo 064) e
      corpo vazio
- [ ] Validação ao vivo: inserir via `/`, escrever no corpo, colapsar,
      salvar, recarregar — conteúdo intacto no `.md`

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Callout aninhado dentro de callout
- Embeds dentro do corpo do callout (o corpo é markdown comum; a
  segmentação de embed roda só no nível da página)
- Cor customizada por hex — as variantes mapeiam pros tokens

## Notas

O corpo guarda markdown, não HTML — é o que garante que o `.md` no
disco continue legível e editável por um agente via CLI.
