---
id: "126"
titulo: "Grafo navegavel por teclado"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["120", "122", "123"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Grafo navegável por teclado

## Objetivo

`graph_view.rs` (ciclos 120/122): os nós são `<g onclick>` SVG — sem
`tabindex`, sem handler de teclado, totalmente invisíveis pro Tab.
Zoom/pan (via mouse) já funcionam; falta o equivalente de teclado pra
abrir uma página do grafo sem mouse.

## Critérios de aceite

- [x] Cada `<g class="graph-view__node">` ganha `tabindex="0"` — vira
      alcançável via Tab, na ordem dos nós (mesma ordem do círculo,
      `2πi/n`)
- [x] Enter/Espaço com um nó focado ativa a navegação (mesmo callback
      do `onclick` já existente) — checa `.key()` ("Enter"/`" "`) E
      `.code() == "Space"` como reforço (ver Notas)
- [x] Indicador de foco visível no nó — `outline` trocado por `stroke`
      no `<circle>` via `:focus-visible`, mais confiável dentro de SVG
      (decisão já prevista neste critério, confirmada necessária)
- [x] Setas do teclado (↑↓←→) pra mover o foco pro nó mais próximo:
      NÃO implementado — Tab/Shift+Tab em ordem do círculo é
      suficiente pra v1, conforme a flexibilidade prevista neste
      próprio critério; heurística de "mais próximo" ficaria
      desproporcional ao valor pro tamanho de vault atual
- [x] `cd ui && cargo test --lib`, `trunk build` passam
- [x] Validação ao vivo via MCP `tauri`: nó focado programaticamente
      (mesma limitação de automação dos ciclos 123/124 pra Tab
      nativo), Enter abriu a página ("missao"), Space (depois do
      reforço `.code()`) abriu outra ("roadmap")

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Navegação de teclado pra fazer PAN/ZOOM (setas movendo a câmera em
  vez do foco) — setas ficam reservadas pra mover o foco entre nós;
  zoom/pan continuam só mouse/botões (ciclo 122)
- Anúncio de acessibilidade via `aria-label`/screen reader — fora do
  escopo definido pro tema (foco é operabilidade via teclado visual,
  não compatibilidade com leitor de tela)

## Notas

Depende do ciclo 123 (regra genérica de `:focus-visible`) já existir,
pra decidir se precisa de CSS específico pro SVG ou se o genérico já
resolve — testar primeiro antes de escrever CSS extra. Resultado:
precisou de CSS específico mesmo (`stroke` no `<circle>` em vez de
`outline` no `<g>`), como o critério já previa como possibilidade.

**Quirk do driver MCP encontrado durante a validação** (não bug do
app): `webview_keyboard press Space` manda `KeyboardEvent.key ===
"Space"` (o nome do código), não `" "` (o caractere literal, que é o
valor correto por spec pra `.key` num navegador de verdade). Meu
handler original só checava `e.key() == " "`, então Space não
disparava durante o teste — reforcei com `e.code() == "Space"`
também (que É "Space" por spec, então cobre tanto navegador real
quanto esse driver). Não muda o comportamento pra um usuário com
teclado físico de verdade, só torna o handler mais tolerante.
