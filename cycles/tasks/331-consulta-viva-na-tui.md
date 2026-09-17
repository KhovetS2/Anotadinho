---
id: "331"
titulo: "A consulta viva na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["327"]
---

# 331 — A consulta viva na TUI

A TUI mostrava só `from pages`. Agora a consulta RODA: o `main` varre o
vault uma vez (`handle_scan_vault`), o índice vai pro estado e cada
consulta da página troca os filhos pelo resultado de
`analise::partes_da_consulta` (cabeçalho com o recorte e a contagem;
resultados com página e campos escondidos; grupos com total e agregados).

Desenho como `.query-embed`: barra com a lupa, o recorte
(`Query::descrever`, agora do núcleo e usado também pela janela) e
"N páginas"; resultados em lista (título e pílulas coloridas pelo valor),
tabela (PÁGINA + uma coluna por campo) ou cartões (grade de quadros com
título, caminho e pílulas); grupos com seta, total e agregados; "Nenhuma
página bate…" quando vazia. `j`/`k` andam nos resultados; `Enter` abre a
página.

Testes de tela com um índice falso e fixo: lista, cartões, agrupada,
vazia e linha acesa.
