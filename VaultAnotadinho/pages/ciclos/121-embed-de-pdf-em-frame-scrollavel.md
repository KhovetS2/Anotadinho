---
title: Ciclo 121 — Embed de PDF em frame scrollavel
type: ciclo
ciclo: "121"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 121 — Embed de PDF em frame scrollavel

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Embed de PDF em frame scrollável

## Objetivo

Hoje um link/imagem markdown apontando pra um `.pdf` em `assets/` não
renderiza nada útil (a tag `<img>` não exibe PDF). Ao encontrar um
link markdown pra um arquivo `.pdf`, o render do editor deve mostrar
um frame próprio, com altura fixa e scroll interno, exibindo o PDF —
sem precisar abrir um app externo.

## Critérios de aceite

- [x] Link markdown `[texto](assets/arquivo.pdf)` vira um `<iframe>`
      dentro de um wrapper com altura fixa (`600px`) e scroll interno,
      em vez de um `<a>` comum — detecção por extensão `.pdf` no href.
      Implementado como pós-processamento DOM depois do
      `set_inner_html` (`upgrade_embedded_assets_at`, `editor.rs`),
      não em `markdown_render.rs` — mantém o parser síncrono
      (`pulldown_cmark`) intocado, a troca `<a>`→iframe acontece na
      árvore já montada, mesmo padrão já usado pro `init_mermaid_at`
- [x] Path relativo do asset resolvido — **descoberta**: não existia
      NENHUM mecanismo de resolução (nem protocolo Tauri, nem
      `convertFileSrc`); `<img src="assets/x.png">` resolvia contra a
      origem do webview (`http://localhost:1420/...`), nunca contra o
      vault — bug pré-existente, imagens NUNCA apareciam de verdade.
      Corrigido com um comando novo (`read_asset_data_url`) que lê o
      arquivo e devolve `data:<mime>;base64,...`; usado tanto pro
      `<iframe>` do PDF quanto pro `<img>` das imagens (mesma correção
      serve os dois)
- [x] `ui/src/html_to_md.rs`: serialização de volta preserva o link
      original (`[texto](arquivo.pdf)`) via novo case `data-pdf-href`
      no branch `"div"`, retornando a forma inline (sem `\n\n`)
- [x] CSS novo `.pdf-embed`/`.pdf-embed__frame`: altura fixa (600px),
      borda consistente com o resto dos embeds
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [x] Validação ao vivo via MCP `tauri`: PDF de teste real (válido,
      gerado à mão) em `assets/`, linkado de uma página — wrapper
      `.pdf-embed` criado, iframe com `data:application/pdf;base64,...`
      correto, webview continuou responsivo, salvar preservou o link
      original exatamente; imagem de teste (PNG 1×1 real) confirmada
      carregando de verdade (`naturalWidth=1`, não mais quebrada)

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

**Bug pré-existente crítico encontrado e corrigido**: nenhuma imagem
embutida via `![]()` jamais apareceu de verdade em nenhum ciclo
anterior — o `src` relativo cru nunca resolvia contra o vault. Ninguém
notou porque a validação anterior sempre checava o HTML gerado
(`outerHTML` correto) ou o arquivo salvo no disco, nunca se a imagem
de fato *carregava* no navegador. Este ciclo corrige isso de quebra
(mesmo mecanismo — `read_asset_data_url` — resolve tanto `<img>`
quanto `<iframe>` de PDF).

**Armadilha encontrada durante a validação**: `webview_screenshot`
(html2canvas) TRAVA o webview inteiro (timeout em qualquer JS
subsequente, inclusive `document.title`) ao tentar capturar uma
página com o iframe de PDF montado — não é um bug do embed em si
(confirmado com um teste isolado: o mesmo iframe, fora do fluxo de
screenshot, funciona e o webview continua responsivo por vários
segundos). Processo de dev precisou ser matado e reiniciado depois.
Lição: NÃO chamar `webview_screenshot` em páginas com embed de PDF —
validar via inspeção de DOM (`img.naturalWidth`, `iframe.src`,
`document.title` respondendo) em vez disso.

Formato final: `read_asset_data_url` (não o protocolo de asset do
Tauri, que exigiria configurar `assetProtocol`/CSP em
`tauri.conf.json` — mais arriscado de acertar às cegas). Uma `data:`
URL resolvida sob demanda, via IPC, cobre imagem e PDF com o mesmo
código.

## Resultado

# Ciclo 121 - done

## Resumo

Link markdown `[texto](assets/x.pdf)` vira um frame com scroll interno
em vez de um `<a>` comum. No processo, encontrado e corrigido um bug
crítico pré-existente: NENHUMA imagem embutida (`![]()`) jamais
resolvia de verdade — `src` relativo cru resolvia contra a origem do
webview, não contra o vault no disco. Novo comando
`read_asset_data_url` corrige os dois de uma vez.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `read_asset_bytes`, 3 testes
- `crates/ipc/src/lib.rs` — `handle_read_asset_data_url` + `guess_mime`,
  3 testes; `crates/ipc/Cargo.toml` ganha dev-dep `tempfile`
- `src-tauri/src/main.rs` — comando Tauri `read_asset_data_url`
- `ui/src/api.rs` — wrapper `read_asset_data_url`
- `ui/src/components/editor.rs` — `upgrade_embedded_assets_at`
  (troca `<a href="*.pdf">` por wrapper `.pdf-embed`, resolve
  `<img>`/`<iframe>` pra `data:` URL), chamada no pipeline de render
- `ui/src/html_to_md.rs` — case `data-pdf-href` reconstrói o link
  original
- `ui/src/styles/main.css` — `.pdf-embed*`

## Testes

`cargo test --workspace`: 116. `cd ui && cargo test --lib`: 79. Total 195.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: PDF real de teste embutido e
renderizado (iframe com `data:application/pdf` correto, webview
responsivo), round-trip de save preservou o link original, e imagem
de teste confirmada carregando de verdade pela primeira vez
(`naturalWidth=1`, não mais quebrada).

## Notas

`webview_screenshot` trava com PDF embutido (bug do html2canvas, não
do embed) — documentado no arquivo de task pra não repetir o erro em
validações futuras.

Fecha a rodada de 8 ciclos (114-121): CLI empacotado no build, CLI
com filtros e escrita, histórico via git, paste de imagem, sync via
git, grafo de backlinks, embed de PDF (+ correção do bug de imagem).
