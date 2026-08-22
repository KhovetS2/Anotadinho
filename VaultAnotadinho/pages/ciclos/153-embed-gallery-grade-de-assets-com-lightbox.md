---
title: "Ciclo 153 — Embed gallery: grade de assets com lightbox"
type: ciclo
ciclo: "153"
status: concluida
date: 2026-08-19
prioridade: media
depende_de: ["148"]
tags:
- ciclo
---

# Ciclo 153 — Embed gallery: grade de assets com lightbox

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Embed gallery: grade de assets com lightbox

## Objetivo

Imagens numa nota hoje entram uma a uma como `<img>` solto (via `/img`
ou colar), empilhadas verticalmente em tamanho cheio. Pra uma nota de
referência visual (moodboard, screenshots de um bug, fotos de uma
sessão) isso é ruim de ler e ocupa a página inteira. Este embed
organiza assets do vault numa grade com legenda e abre em tamanho
grande ao clicar.

## Critérios de aceite

- [x] `EmbedKind::Gallery` + `{{ type: "gallery" }}`
- [x] `GalleryEmbedData { columns: u8 (default 3), size: GallerySize
      (Sm|Md|Lg, default Md), items: Vec<GalleryItem { path, caption }> }`
- [x] Componente `embeds/inline_gallery.rs`: grade responsiva,
      `object-fit: cover`, legenda editável inline abaixo de cada item
- [x] Botão "adicionar do vault" abre o picker de assets — mesma
      chamada `api::list_assets_info` usada pelo `__ASSET__` do menu
      `/` (editor.rs), sem duplicar a lógica de listagem
- [x] Paths `assets/...` resolvidos pra data URL por
      `api::read_asset_data_url` (a mesma travessia que o
      `upgrade_embedded_assets_at` do editor faz, mas chamada direto:
      aquele helper reescreve DOM injetado por `set_inner_html`, e aqui
      os `<img>` são do VDOM do Yew). Resultado memorizado num mapa
      path → data URL, resolvido sob demanda; URL externa (`http...`)
      renderiza direto
- [x] Clique abre lightbox sobre `components/modal.rs` (trap de foco e
      Escape já vêm de lá), com setas ←/→ pra navegar entre os itens
- [x] Remover item e reordenar (mover pra esquerda/direita)
- [x] Item cujo arquivo não existe mais renderiza um placeholder com o
      path, não quebra a grade
- [x] `data-nav-item`/`data-nav-group`; Enter no item abre o lightbox
- [x] Testes de round-trip: legenda com vírgula/dois-pontos, lista
      vazia, `columns` fora do intervalo 1-6 normalizado

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Upload por drag-and-drop de arquivo do SO pra dentro da galeria — o
  fluxo de entrada de asset continua sendo colar (ciclo 118) ou
  `/img`; aqui só se escolhe o que já está em `assets/`
- Edição de imagem (crop, rotate)
- Vídeo/áudio embutido

## Notas

`cargo test -p anotadinho-core`: 119 (115 + 4 novos). `cargo test
--workspace`, `cd ui && cargo test --lib` (26), `trunk build`, `cargo
build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo (MCP `tauri`): duas imagens copiadas temporariamente
pra `assets/` (o vault de exemplo só tinha um PDF). Inserida por
`/galeria`, nasce vazia com a explicação; picker listou só as imagens
(o PDF ficou de fora); as duas entraram e carregaram como data URL;
legenda com dois-pontos gravada; lightbox abriu, navegou pra próxima e
fechou no Escape; salvo e conferido no disco — `caption: 'Estado
atual: v1'` escapado pelo serde, legenda vazia não serializada. As
imagens temporárias foram removidas do vault no fim.

Ícone novo em `icon.rs`: `image` (e `plus` se ainda não existir).

## Resultado

# Ciclo 153 - done

## Resumo

`{{ type: "gallery" }}` — grade de imagens do vault com legenda,
número de colunas e tamanho de miniatura configuráveis, reordenar,
remover e lightbox com navegação entre os itens. Fecha a leva de
embeds de composição (callout, columns, gallery).

## Arquivos criados/modificados

- `crates/core/src/embed.rs` — `EmbedKind::Gallery`, `GallerySize`,
  `GalleryItem`, `GalleryEmbedData` + 4 testes
- `crates/core/src/index.rs` — braço do novo tipo
- `ui/src/components/embeds/inline_gallery.rs` (novo)
- `ui/src/components/embeds/mod.rs` — registro + dispatcher
- `ui/src/styles/main.css` — `.gallery*`

## Testes adicionados

- round-trip com legenda cheia de pontuação e item externo (URL)
- `columns` fora de 1..6 normalizado (0 → 1, 99 → 6, ausente → 3)
- mover item nas pontas é no-op
- legenda vazia não é serializada

## Problemas encontrados

- `upgrade_embedded_assets_at` (o helper do editor) não serve aqui: ele
  reescreve DOM injetado por `set_inner_html`, e os `<img>` da galeria
  são do VDOM do Yew. A resolução usa `api::read_asset_data_url`
  direto, com memoização num mapa por path.
- Item cujo arquivo sumiu mostra o path como placeholder — sem isso a
  grade ficava com um buraco e nenhuma pista do que consertar.

## Notas para próximos ciclos

- Entrada de arquivo continua sendo colar (118) ou `/img`; a galeria só
  escolhe o que já está em `assets/`.
- Próximo bloco é o de dados vivos: 154 (query) usa o `scan_vault` do
  ciclo 150.
