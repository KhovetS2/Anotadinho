---
id: "326"
titulo: "As colunas lado a lado na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["325"]
estima_min: 60
---

# 326 — As colunas lado a lado na TUI

Antes os painéis saíam um embaixo do outro, com o nome `pane` vazando.
Agora, como `.columns-embed__grid`:

- Os painéis LADO A LADO, cada um com a fração da largura do arquivo
  (`width: 2` → `2fr`). Cada painel é um quadro com o fundo da página por
  dentro, a largura `Nfr` em cima (a barra que a janela mostra no foco) e
  o markdown com o mesmo desenho da página, quebrado na largura do painel.
- Os quadros têm a altura do mais alto. O painel com o cursor acende a
  borda; a linha do cursor lá dentro acende como no resto da página.
- `h`/`l` trocam de painel; `Enter` entra, `j`/`k` andam no markdown.

No núcleo, cada painel carrega a `largura` escondida.
