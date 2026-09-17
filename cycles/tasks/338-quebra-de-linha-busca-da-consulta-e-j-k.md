---
id: "338"
titulo: "Quebra de linha, busca da consulta e J/K pra reordenar"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["337"]
---

# 338 — Quebra de linha, busca da consulta e `J`/`K`

- Blocos de texto QUEBRAM na largura em vez de cortar; as quebras de um
  parágrafo de várias linhas viram linhas; a continuação de item alinha
  depois da marca.
- A barra da consulta virou parte `busca` (não mais `cabecalho`): o
  cursor pousa nela e `Enter` edita o filtro — antes ela não era
  alcançável. A dica "↵ filtrar" substitui o ✲.
- `J`/`K` (com contagem no `K`) reordenam na vertical: cartão dentro da
  coluna, linha da tabela, barra do cronograma, bloco de markdown. Nas
  listas horizontais (botões, imagens, painéis, colunas) são o mesmo que
  `>>`/`<<`. Na gramática: `K` → `Comando::Subir`, `J` (juntar linhas)
  vale `Reordenar(1)` num item; a janela ignora `K`.
