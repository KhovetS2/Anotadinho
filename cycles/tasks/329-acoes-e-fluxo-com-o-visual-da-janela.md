---
id: "329"
titulo: "Ações e fluxo com o visual da janela na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["327"]
---

# 329/330 — Ações e fluxo com o visual da janela

Comparados com prints do DOM vivo da janela:

- Ações: botão comum em `--bg-surface` com texto normal
  (`.actions-embed__btn`), primário em `--accent-blue`, ícone do YAML como
  glifo de uma célula, e o "+ ação" apagado no fim. Sob o cursor, o
  contorno acende na cor de destaque e o preenchimento fica.
- Fluxo: topo com o artefato apagado e a etapa em pílula na cor dela; a
  trilha em pílulas feita/atual/futura, sem "Bloqueada" (a não ser que
  seja a atual); "Pedir alteração"/"Executar" em botão elevado; transição
  principal cheia (azul→roxo), as outras sem caixa (`.btn--ghost`); dica
  e nota apagadas.
