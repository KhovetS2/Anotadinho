---
id: "151"
titulo: "Embed callout: caixa de destaque colapsável"
status: done
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

- [x] `EmbedKind::Callout` + `{{ type: "callout" }}` reconhecido por
      `segment`
- [x] `CalloutEmbedData { variant: CalloutVariant (Info|Success|
      Warning|Error|Tip), title: String, collapsed: bool, body: String }`
      com parse/serialize por derive de serde (nunca montando string)
- [x] Componente `embeds/inline_callout.rs`: header com ícone da
      variante + título editável inline + botão de colapsar; corpo
      renderizado por `markdown_render::render`
- [x] `ui/src/components/embeds/markdown_field.rs` (novo):
      componente `EmbedMarkdownField`, com DOIS estados em vez do
      `contenteditable` planejado — lendo (`<div>` com o HTML
      renderizado por `set_inner_html`) e editando (`<textarea>` com o
      markdown cru, autogrow, commit no blur ou Ctrl+Enter, Escape
      cancela). Ver Notas: contenteditable aqui reintroduziria o bug do
      ciclo 076. NÃO participa de `segment_refs`: a mutação viaja pelo
      `on_change` do embed, igual kanban/tabela
- [x] Trocar a variante por um seletor no header (sem modal)
- [x] Ícones novos em `icon.rs`: `info`, `alert-triangle`,
      `alert-circle`, `lightbulb` e `chevron-down` (o `check` já existe
      pra Success)
- [x] CSS BEM `.callout` / `.callout--{variant}` só com tokens; fundo
      via `color-mix(in srgb, var(--accent-...) 12%, transparent)`
- [x] `data-nav-item`/`data-nav-group` no header e nos controles, foco
      visível
- [x] Testes de round-trip: parse(serialize(x)) == x, incluindo corpo
      multi-linha com `:` e `#` (o caso que quebrou no ciclo 064) e
      corpo vazio
- [x] Validação ao vivo: inserir via `/`, escrever no corpo, colapsar,
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

`cargo test -p anotadinho-core`: 110 (105 + 5 novos de callout).
`cd ui && cargo test --lib`: 26. `cargo test --workspace`, `trunk
build`, `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

**Decisão de implementação (diferente do planejado):** o corpo NÃO usa
`contenteditable`. O ciclo 076 já documentou o modo de falha — um nó de
texto dentro de um `contenteditable` que o Yew re-renderiza a cada
mudança do embed deixa o VDOM apontando pra nó desatualizado, e a
quebra de linha que o WebKit insere sozinho vira texto duplicado que
nunca é reconciliado. O `EmbedMarkdownField` usa a mesma solução que a
célula Text da tabela: `<textarea>` (valor é propriedade do elemento,
não filhos de DOM) enquanto edita, HTML renderizado enquanto lê.
Consequência: `html_to_md::html_to_markdown` não é usado aqui — o
markdown nunca vira HTML editável, então não precisa voltar.

Parse tolerante: `variant` deserializa por função própria que aceita
qualquer string e cai no default. Sem isso um `variant: roxo` (typo, ou
versão futura, ou agente escrevendo pelo CLI) derrubava a struct
inteira via `unwrap_or_default` — e a primeira regravação apagaria
título e corpo do usuário. Tem teste de regressão.

Validação ao vivo (MCP `tauri`): inserido por `/destaque`, trocado pra
variante Atenção, corpo editado com `**negrito**`, `` `codigo` ``,
dois-pontos e lista, salvo e RECARREGADO do disco — voltou idêntico. O
`.md` fica legível (`body: |` com bloco indentado), que é o que permite
um agente editar por fora. Recolher/expandir OK.

O corpo guarda markdown, não HTML — é o que garante que o `.md` no
disco continue legível e editável por um agente via CLI.
