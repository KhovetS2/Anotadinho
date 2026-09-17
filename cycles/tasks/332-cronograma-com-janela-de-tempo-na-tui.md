---
id: "332"
titulo: "O cronograma com a janela de tempo da tela na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["327", "331"]
---

# 332 — O cronograma com a janela de tempo na TUI

Antes a janela era "do primeiro início ao último fim" e o modo vault era
ignorado. Agora é a da janela gráfica:

- A janela tem os dias da escala (Semana 7, Mês 35, Trimestre 91) e, sem
  âncora, começa um quarto antes de hoje. `[`/`]` andam uma janela, `t`
  volta pra hoje, `m` troca a escala (gravada no arquivo, como na janela)
  e `~` alterna Manual/Vault (gravado).
- Cabeçalho como `.timeline__bar`: período e escala, "Hoje", seletor de
  escala com a ativa cheia, fonte e "+ etapa" (manual) — com as teclas.
- Eixo `dd/mm` por semana (por dia na Semana) e hoje cruzando a régua.
- Barras de UMA linha, como `.timeline__bar-item`: pílula na cor do badge,
  título cortado, hoje atravessando o trilho. Barra fora da janela some.
- Modo vault: páginas com `start`/`date` (até `end`/`due`) viram barras
  (`calendario::itens_do_vault`); Enter abre a página.

Núcleo: `analise::partes_do_cronograma(dados, janela, hoje)`; sem janela
continua o comportamento do CLI.
