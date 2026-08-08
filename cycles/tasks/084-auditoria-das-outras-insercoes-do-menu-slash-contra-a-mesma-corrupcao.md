---
id: "084"
titulo: "Auditoria das outras insercoes do menu slash contra a mesma corrupcao"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: ["079", "082"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Auditoria das outras inserções do menu slash contra a mesma corrupção

## Objetivo

O ciclo 079 trocou `execCommand("insertHTML", ...)` por inserção via
`Range` só pros 3 itens de embed do menu `/`, já que o `execCommand`
demonstrou fragmentar HTML de forma imprevisível no WebKitGTK dependendo
de onde o cursor estava. Os outros itens (título, lista, checklist,
citação, código, linha, tabela markdown, imagem, diagrama, assets)
continuavam usando o caminho antigo, com o mesmo risco (menos
catastrófico que embed, mas ainda real: um heading inserido dentro de um
item de lista virava `- # Título` — texto literal, não um heading de
verdade).

## Critérios de aceite

- [x] `insert_embed_marker_at_cursor` refatorado em cima de um
      `insert_element_at_cursor` genérico, reaproveitável por qualquer
      item do menu (não só embed)
- [x] Item "catch-all" (título, lista, checklist, citação, código, linha,
      tabela markdown) usa `insert_element_at_cursor(el, true)` — quebra
      pra fora de `<li>`/`<p>`/blockquote/heading, igual embed
- [x] Imagem/Assets usam `insert_element_at_cursor(el, false)` — NÃO
      quebra pra fora do bloco (imagem/link são conteúdo inline-safe,
      faz sentido continuar dentro do parágrafo)
- [x] Diagrama (mermaid) usa `insert_element_at_cursor(el, true)` —
      bloco, igual embed
- [x] `mark_edited` chamado explicitamente depois de cada inserção bem
      sucedida — `Range.insertNode`/`Node.insertBefore` não disparam
      `oninput` sozinhos (diferente de `execCommand`), então sem isso o
      autosave nunca saberia que algo mudou

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Resolver a fragilidade de posição do cursor nos fluxos com diálogo
  (`Imagem`/`Assets`/`Diagrama` abrem um `Prompt` antes de inserir — o
  foco muda pro campo do diálogo nesse meio tempo, então a seleção
  original no editor pode não sobreviver) — é uma limitação
  pré-existente, não piorada por este ciclo, e resolvê-la de vez exigiria
  guardar/restaurar a seleção explicitamente, fora de escopo aqui

## Notas

**Bug real (menor que o de embed, mas real) confirmado e corrigido**:
inserir "Título 1" com o cursor dentro de um item de lista vazio, ANTES
deste ciclo, produzia `- # Título` no markdown final — texto literal
começando com `#`, NÃO um heading de verdade (o parser de markdown não
reconhece `#` no meio de um item de lista como heading). Mesmo problema
pra Linha (`<hr>` virava `- ---`), Citação, Código, Tabela markdown.

**Fix**: generalizado o mecanismo do ciclo 079
(`insert_embed_marker_at_cursor` → `insert_element_at_cursor`, que
recebe qualquer `Element` em vez de só o marcador de embed) e aplicado
em TODOS os itens do menu `/`, com `break_out_of_block` decidido por
tipo de conteúdo (bloco vs inline-safe).

De passagem, adicionado `parse_single_element(html) -> Option<Element>`
(constrói um elemento a partir de uma string HTML via um `<div>`
wrapper temporário — todos os itens do menu produzem exatamente UM
elemento raiz, então isso é suficiente).

Validado ao vivo via MCP `tauri` na página `teste`: `/linha` (cursor
dentro de `<li>` vazio) → `<hr>` inserido como IRMÃO de `<ul>` (não
aninhado), salvar confirma `---` sozinho na linha no arquivo (era
`- ---` antes do fix). `/tulo` → "Título 1" → `<h1>Título</h1>` também
corretamente como irmão da lista, não aninhado dentro do `<li>`.
