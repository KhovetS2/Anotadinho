---
title: Revisar proposta vendo a mudança e a página final
type: spec
date: 2026-10-07
status: rascunho
prioridade: alta
tags:
- spec
- propostas
- tui
---
{{ type: "fluxo" }}
artefato: spec
etapa: rascunho
{{ /fluxo }}
## Por que isto existe

A revisão é a defesa que continua valendo mesmo quando o agente erra: a
mudança só vira arquivo depois de alguém ver e aprovar. Ela tem dois
modos, e hoje nenhum dos dois mostra o que promete.

Medido em 2026-10-07, na janela e na TUI:

**Visualização na janela não é a tela final.** Ela usa o mesmo
renderizador de markdown do editor, mas fora do contêiner do editor, e
por isso não pega o estilo da página:

| | Visualização | Página no editor |
|---|---|---|
| Texto | 16px / 24px | 15px / 25,5px |
| `h2` | peso 700, sem margem | peso 600, 16px acima |
| Parágrafo | sem margem | 8px abaixo |
| Lista | sem recuo | recuo de 24px |
| `code` | sem fundo | fundo |
| `**negrito**` | cor do texto | cor de destaque |

Também ficam de fora os passos que o editor aplica depois de renderizar:
imagens do vault, transclusão `![[...]]`, realce de código e mermaid.

**Visualização ignora os trechos tirados.** Nos dois lados ela mostra a
proposta INTEIRA, mesmo com trechos desmarcados. "Aplicar 1 de 3" grava
uma página que a Visualização não mostrou.

**Diff da TUI esconde a mudança.** A linha é cortada na borda. Numa
linha de markdown longa, as versões `-` e `+` aparecem idênticas, porque
o que mudou ficou depois do corte. O cabeçalho ("Visualizaç") e a linha
de atalhos também são cortados.

**Diff da janela esconde a mudança até rolar.** A linha não quebra
(`white-space: pre`). A mudança no fim de uma linha longa só aparece
rolando a caixa pro lado.

## Requisitos funcionais

1. **RF1.** Na janela, a Visualização desenha a página como o editor a
   desenha: mesma tipografia, espaçamento, listas, checkboxes, código,
   links, imagens do vault, transclusões, realce e diagramas.
2. **RF2.** Nos dois lados, a Visualização mostra a página que SERÁ
   gravada: com trechos tirados, o resultado sem eles.
3. **RF3.** Nos dois lados, a linha longa do Diff quebra em vez de ser
   cortada ou rolar. Uma mudança em qualquer ponto da linha fica à
   vista, com o realce palavra a palavra.
4. **RF4.** Na TUI, o cabeçalho do cartão e a linha de atalhos não
   perdem texto quando não cabem.
5. **RF5.** A Visualização continua somente leitura: nada nela escreve
   no vault nem abre ferramenta de edição.

## Critérios de aceite

- [ ] Janela: na Visualização, `h2`, parágrafo, lista e `code` têm o
      estilo calculado do editor (cenário de harness comparando os dois)
- [ ] Janela: imagem de `assets/` aparece na Visualização
- [ ] Janela e TUI: com um trecho tirado, a Visualização não mostra o
      texto dele
- [ ] TUI: numa linha de 160 caracteres com a mudança no fim, a palavra
      mudada aparece no Diff, num cartão de 60 colunas
- [ ] Janela: a linha longa do Diff quebra, sem rolagem horizontal
- [ ] TUI: "Visualização" aparece inteiro no cabeçalho, em 60 colunas

## Ambiguidades (precisam de decisão)

- **Marcar na Visualização o que mudou?** Destacar os blocos alterados
  responderia "o que mudou" e "como fica" numa tela só. Não está nos
  requisitos, porque é uma decisão de produto: a Visualização deixa de
  ser "a tela final" pura.

## Fora de escopo

- Visualização lado a lado (antes | depois)
- Editar a proposta pela Visualização
