---
id: "153"
titulo: "Embed gallery: grade de assets com lightbox"
status: pending
criado: 2026-08-19
autor: humano
prioridade: media
depende_de: ["148"]
estima_min: 75
agente_alvo: claude-sonnet
---

# Embed gallery: grade de assets com lightbox

## Objetivo

Imagens numa nota hoje entram uma a uma como `<img>` solto (via `/img`
ou colar), empilhadas verticalmente em tamanho cheio. Pra uma nota de
referência visual (moodboard, screenshots de um bug, fotos de uma
sessão) isso é ruim de ler e ocupa a página inteira. Este embed
organiza assets do vault numa grade com legenda e abre em tamanho
grande ao clicar.

## Critérios de aceite

- [ ] `EmbedKind::Gallery` + `{{ type: "gallery" }}`
- [ ] `GalleryEmbedData { columns: u8 (default 3), size: GallerySize
      (Sm|Md|Lg, default Md), items: Vec<GalleryItem { path, caption }> }`
- [ ] Componente `embeds/inline_gallery.rs`: grade responsiva,
      `object-fit: cover`, legenda editável inline abaixo de cada item
- [ ] Botão "adicionar do vault" abre o picker de assets — mesma
      chamada `api::list_assets_info` usada pelo `__ASSET__` do menu
      `/` (editor.rs), sem duplicar a lógica de listagem
- [ ] Paths `assets/...` resolvidos pra data URL por
      `upgrade_embedded_assets_at` (mesmo caminho do editor); URL
      externa (`http...`) renderiza direto
- [ ] Clique abre lightbox sobre `components/modal.rs` (trap de foco e
      Escape já vêm de lá), com setas ←/→ pra navegar entre os itens
- [ ] Remover item e reordenar (mover pra esquerda/direita)
- [ ] Item cujo arquivo não existe mais renderiza um placeholder com o
      path, não quebra a grade
- [ ] `data-nav-item`/`data-nav-group`; Enter no item abre o lightbox
- [ ] Testes de round-trip: legenda com vírgula/dois-pontos, lista
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

Ícone novo em `icon.rs`: `image` (e `plus` se ainda não existir).
