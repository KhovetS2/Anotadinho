---
id: "122"
titulo: "Zoom e pan interativo no grafo de backlinks"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["120"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Zoom e pan interativo no grafo de backlinks

## Objetivo

O grafo (`type: graph`, ciclo 120) foi entregue com SVG estático — sem
zoom nem pan, listado explicitamente como Não-objetivo na época
("SVG estático que cabe no viewport"). Usuário reportou que isso
limita ver os detalhes num vault com mais páginas. Este ciclo adiciona
zoom (roda do mouse + botões) e pan (arrastar).

## Critérios de aceite

- [x] Zoom via roda do mouse (`onwheel`) sobre o SVG, clampado entre
      25% e 400%
- [x] Botões "−"/"+"/"Reset" na toolbar, mesmo range de zoom, "Reset"
      volta pra 100% e pan zerado
- [x] Pan via arrastar (mousedown+mousemove+mouseup/mouseleave) — 1px
      de mouse = 1px de tela, independente do zoom atual (translate
      fica FORA do scale na composição do `transform` CSS)
- [x] Clicar um nó continua abrindo a página normalmente, mesmo depois
      de zoom/pan (mousedown pra iniciar drag não interfere com o
      onclick nativo do nó quando não há movimento entre down/up)
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [x] Validação ao vivo via MCP `tauri`: zoom in/out via botão E via
      roda do mouse conferidos (100%→156%→125%→138%), pan conferido
      (drag de (100,100) pra (150,130) moveu o `translate` exatamente
      50px/30px), reset conferido, clique em nó depois de zoom/pan
      continuou navegando certo
- [x] Arrastar pra fazer pan não seleciona texto dos rótulos por baixo
      do cursor — `preventDefault()` no `mousedown` + `user-select:
      none`/`-webkit-user-select: none` no `.graph-view__svg` (achado
      do usuário ao testar, corrigido no mesmo ciclo)

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Zoom ancorado no cursor (zoom-to-point) — v1 ancora sempre no centro
  lógico do layout (400,400 no viewBox), via `transform-origin` CSS;
  zoom-to-cursor exigiria recalcular o pan a cada zoom pra manter o
  ponto sob o cursor fixo, complexidade maior sem ganho claro pro
  tamanho de vault atual
- Suporte a touch/pinch-to-zoom — só mouse (roda + arrastar) e botões
  nesta versão
- Persistir zoom/pan entre sessões (lembrar o nível de zoom ao
  reabrir a página) — sempre reseta pra 100%/centro ao trocar de
  página

## Notas

`translate` numa composição CSS `transform: translate(...) scale(...)`
fica FORA da escala (aplicado no espaço do elemento PAI, não no espaço
já escalado) — por isso o delta do mouse em pixels de tela mapeia
direto pro pan sem precisar dividir pela escala atual, simplificando
o handler de `mousemove`.

`WheelEvent` precisou ser adicionado à lista de features do `web-sys`
em `ui/Cargo.toml` — primeiro uso desse tipo de evento no projeto.

Achado durante a validação (não é bug, é timing normal do Yew):
chamar `.click()` duas vezes seguidas no MESMO tick de script e ler o
estado logo em seguida não reflete a atualização — Yew re-renderiza
de forma assíncrona (microtask), então o estado só aparece atualizado
na PRÓXIMA chamada de `webview_execute_js`. Validações futuras de
estado após clique devem sempre separar o clique da leitura em
chamadas distintas.

Seleção de texto durante o drag foi reportada pelo usuário depois da
primeira entrega deste ciclo — corrigida na mesma sessão, sem virar
ciclo separado (`preventDefault` no `mousedown` bloqueia o navegador
de começar a seleção nativa; `user-select: none` no CSS é reforço
defensivo pro caso do WebKitGTK se comportar diferente).
