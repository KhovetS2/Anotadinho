---
id: "121"
titulo: "Embed de PDF em frame scrollavel"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: []
estima_min: 75
agente_alvo: claude-sonnet
---

# Embed de PDF em frame scrollável

## Objetivo

Hoje um link/imagem markdown apontando pra um `.pdf` em `assets/` não
renderiza nada útil (a tag `<img>` não exibe PDF). Ao encontrar um
link markdown pra um arquivo `.pdf`, o render do editor deve mostrar
um frame próprio, com altura fixa e scroll interno, exibindo o PDF —
sem precisar abrir um app externo.

## Critérios de aceite

- [ ] `ui/src/markdown_render.rs`: link markdown `[texto](arquivo.pdf)`
      (ou `assets/arquivo.pdf`) vira um `<iframe>`/`<embed>` dentro de
      um wrapper com altura fixa (`~600px`) e `overflow-y: auto`, em
      vez de um `<a>` comum — detecção por extensão `.pdf` no href
- [ ] Path relativo do asset resolvido pro protocolo de asset do Tauri
      (mesmo mecanismo já usado pras imagens — verificar em
      `markdown_render.rs`/`embed.rs` como `![]()` resolve hoje o path
      de `assets/` e reaproveitar)
- [ ] `ui/src/html_to_md.rs`: serialização de volta preserva o link
      original (`[texto](arquivo.pdf)`) — não pode virar embed fence
      nem perder o formato ao salvar (ligado ao bug corrigido no
      ciclo 111, cuidado extra de round-trip aqui)
- [ ] CSS novo pro wrapper (`.pdf-embed` ou similar): altura fixa,
      scroll interno, borda consistente com o resto dos embeds
      (mesmo padrão visual do `.editor__wysiwyg` pra tabelas/código)
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [ ] Validação ao vivo via MCP `tauri`: colocar um `.pdf` de teste em
      `assets/`, linkar de uma página, confirmar que renderiza dentro
      de um frame com scroll (não abre em nova janela/app externo), e
      que salvar a página não corrompe o link

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Preview de PDF em miniatura (thumbnail) na listagem de assets
  (ciclo 096) — só o embed dentro da página
- Suporte a outros formatos de arquivo embutido (vídeo, áudio) — só
  PDF nesta versão; o padrão fica fácil de estender depois
- Anotação/highlight dentro do PDF — é só visualização, sem interação
  além de scroll (o viewer nativo do WebView já dá zoom/busca de
  graça, isso não precisa ser reimplementado)

## Notas

Verificar primeiro COMO `![alt](assets/img.png)` resolve o `src` real
hoje (provavelmente via `convertFileSrc`/protocolo customizado do
Tauri, não um path relativo cru) — o embed de PDF precisa do mesmo
mecanismo de resolução de path, só trocando a tag final
(`<img>` → `<iframe>`/`<embed>`).
