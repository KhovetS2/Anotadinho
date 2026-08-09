---
id: "126"
titulo: "Grafo navegavel por teclado"
status: pending
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

- [ ] Cada `<g class="graph-view__node">` ganha `tabindex="0"` — vira
      alcançável via Tab, na ordem dos nós (mesma ordem do círculo,
      `2πi/n`)
- [ ] Enter/Espaço com um nó focado ativa a navegação (mesmo callback
      do `onclick` já existente)
- [ ] Indicador de foco visível no nó (ciclo 123 cobre o caso genérico
      via CSS, mas `outline` em elementos SVG tem suporte inconsistente
      entre navegadores — se `:focus-visible` com `outline` não
      aparecer direito no WebKitGTK, usar alternativa SVG-nativa: ex.
      `.graph-view__node:focus-visible circle { stroke: var(--accent-blue);
      stroke-width: 3; }`)
- [ ] Setas do teclado (↑↓←→) movem o foco pro nó mais próximo nessa
      direção (opcional/bom-ter — se a heurística de "mais próximo"
      ficar complicada demais, Tab/Shift+Tab em ordem já é aceitável
      pra v1; documentar a decisão tomada)
- [ ] `cd ui && cargo test --lib`, `trunk build` passam
- [ ] Validação ao vivo via MCP `tauri`: Tab até um nó do grafo,
      confirma foco visível, Enter abre a página

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
resolve — testar primeiro antes de escrever CSS extra.
