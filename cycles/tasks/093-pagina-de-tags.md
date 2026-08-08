---
id: "093"
titulo: "Pagina de Tags"
status: done
criado: 2026-08-07
autor: humano
prioridade: baixa
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Página de Tags

## Objetivo

Oitavo ciclo do conjunto grande. Página `type: tags` agregando todas as
tags usadas no vault (cards de kanban + eventos de calendário) e as
páginas onde cada uma aparece, mesmo espírito somente-leitura/navegação
da página `type: calendar` (ciclo 085).

## Critérios de aceite

- [x] `ui/src/embed.rs`: `scan_vault_tags` — escaneia todas as páginas,
      agrega tag → `Vec<(page_path, page_title)>`, dedup por página
      (mesma tag em 2 cards da MESMA página conta 1x)
- [x] `ui/src/components/tags_page.rs` novo: dispatchado por
      `page_view.rs` (`"tags" =>`), lista tags com contagem + chips de
      página clicáveis (navega via `on_page_selected`)
- [x] Comando "Ver Tags" na paleta (Ctrl+K) — abre a página `pages/tags.md`
      se já existir, senão cria (`type: tags`) — evita duplicar a página
      a cada clique
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
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

- Tags de colunas Select/MultiSelect da tabela embed — extrair exigiria
  saber QUAL coluna é "tag-like" (não tem convenção fixa, cada tabela
  define suas próprias colunas); só kanban (`card.tags`) e calendário
  (`entry.all_tags()`) entram nesta v1, ambos com um campo de tags
  estruturado de verdade
- Tags de frontmatter de página (`tags: [...]` no YAML do topo) — são
  metadados da página em si, semanticamente diferentes das tags de
  ITENS dentro de embeds; ver Notas
- Filtrar/renomear tags em massa a partir da página — só visualização e
  navegação

## Notas

Frontmatter `tags: [...]` (ex: `VaultAnotadinho/pages/sobre.md`) e tags
de embed (`card.tags`/`entry.tags`) são conceitos DIFERENTES apesar do
nome igual — o primeiro categoriza a PÁGINA inteira, o segundo
categoriza um ITEM dentro dela (um card, um evento). `scan_vault_tags`
só olha o segundo tipo. Se no futuro fizer sentido unificar os dois na
mesma página de Tags, é ciclo futuro.

Validado ao vivo via MCP `tauri`: `Ctrl+K` → "Ver Tags" cria
`pages/tags.md` (`type: tags`) e navega; página mostra "bug"/"infra"/
"urgente" (vindas de `exemplos-embeds.md`: tag de card kanban + `tag:`
legado de 2 eventos de calendário), cada uma com 1 página listada;
clicar o chip "exemplos-embeds" navega pra lá. Confirmado que colunas
Select/MultiSelect da tabela embed (mesmo arquivo, valores
"urgente, bug"/"infra") ficam de fora, como esperado. Página de teste
removida (`rm VaultAnotadinho/pages/tags.md`) antes de fechar o ciclo.
