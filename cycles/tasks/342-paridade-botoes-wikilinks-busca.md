---
id: "342"
titulo: "Paridade: executar botões, seguir wikilinks e buscar no conteúdo"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["341"]
---

# 342 — Executar botões, seguir wikilinks e buscar no conteúdo

Primeiro ciclo da série de paridade janela × TUI. O inventário lado a lado
está em `docs/paridade-gui-tui.md` e é atualizado a cada ciclo.

- Botões de ação executam no `Enter`, como `run_action` da janela: abrir
  página, nova página a partir de template (pede o título; pedido
  `CriarDeTemplate`), gravar propriedade no frontmatter
  (`set_frontmatter_field`, pedido `DefinirPropriedade`) e busca (abre a
  barra com o termo). Botão sem destino avisa.
- `Enter` num bloco com `[[wikilink]]` abre a página (pelo título do
  frontmatter ou pelo nome do arquivo); com vários, pergunta qual; o que
  não existe se oferece pra criar.
- Busca no conteúdo: na barra de comandos, `Ctrl+F` (ou `Enter` sem
  resultado) roda a busca de texto do vault (`handle_search_content`) e
  mostra página, origem e trecho; `Enter` abre.
- `Modal::Entrada` ganhou `AcaoDaEntrada` (página nova ou de template).
