---
id: "304"
titulo: "Cada embed sua cor: moldura por tipo, cartão do kanban e miniatura da galeria"
status: done
criado: 2026-09-16
autor: agente
prioridade: alta
depende_de: ["302"]
estima_min: 180
---

# 304 — Cada embed sua cor

## O que motivou

O pedido foi montar os embeds que ainda faltavam na TUI — nove dos dez
(todos menos o fluxo, que o ciclo 302 já tinha vestido) — usando
quadrados e retângulos preenchidos com cor pra ajudar a enxergar a
montagem, no mesmo espírito da janela, e puxando as cores do TEMA (pra
continuarem atualizáveis por quem usa) em vez de cravar hex no desenho.

Nove embeds de uma vez é grande demais pra um ciclo só e ainda sair bem
testado. Este ciclo é o primeiro corte, coerente sozinho: toda moldura
de embed ganha a cor do PRÓPRIO tipo, e dois deles — kanban e galeria —
ganham conteúdo em retângulo/quadrado preenchido, não só a moldura. Os
outros sete (calendário, tabela, colunas, consulta, cronograma) ficam
com a moldura colorida e o conteúdo em texto — que já era o estado
anterior, só que agora cada um com a própria cor em vez de todos no
roxo genérico do embed.

## O que mudou

**A moldura pinta pelo TIPO, não mais uma cor só.** `regiao()`
devolvia `Realce::Embed` (roxo) pra QUALQUER embed — seis embeds
vizinhos de tipos diferentes saíam todos com a mesma cor de caixa.
`papel_do_embed(nome)` resolve uma de nove `Realce` novas
(`EmbedKanban`, `EmbedCalendar`, `EmbedTable`, `EmbedCallout`,
`EmbedColumns`, `EmbedGallery`, `EmbedQuery`, `EmbedTimeline`,
`EmbedActions`), cada uma um token que já existia no CSS da janela
(`--cor-azul`, `--cor-verde`, `--warning`…) — nenhuma cor nova, só um
papel novo pra uma que já era do tema. O rótulo fechado (`[kanban]`)
usa a mesma função, então abrir e fechar não troca de cor. O fluxo
fica de fora — já tinha a dele desde o ciclo 293.

**O cartão do kanban e a miniatura da galeria viram caixa preenchida.**
Até aqui só um botão (fileira, lado a lado) ganhava o contorno de
meio-bloco do ciclo 302. Um cartão de kanban é filho de COLUNA — `j`/`k`
andam entre cartões, não fileira — e mesmo assim queria a mesma caixa
fechada. `linha_de_caixa` é o mesmo contorno/miolo do botão, só que
desenhado sozinho na própria linha em vez de lado a lado. O cartão
ESTICA até a borda da coluna (como na janela); a miniatura só cresce
até caber a legenda, como um selo — é a diferença entre "retângulo" e
"quadrado" que o pedido separou.

Do lado do modelo (`crates/core/src/analise.rs`), a galeria trocou o
nome da parte de `"item"` pra `"miniatura"`: o terminal não desenha a
imagem, e o nome é o gancho que o desenho usa pra saber que ali era pra
ser uma. `"card"` e `"miniatura"` entraram em `PARTES_SEM_ROTULO`: a
caixa já diz o que é, repetir em texto seria a mesma informação duas
vezes — a mesma decisão que "titulo"/"acao" já tinham levado no 302.

**O botão de ações se separa em `button` e `button-primary`.**
`variant: primary` já era lido pela janela pra pintar
`.actions-embed__btn--primary` de destaque; o terminal jogava o campo
fora e todo botão saía igual. Agora o nome da parte carrega a
distinção, e só o primário ganha a cor de destaque — o padrão sem
variante fica no `Parte` genérico de sempre, sem mudar de cor à toa.

## O que NÃO entrou

Calendário, tabela, colunas, consulta e cronograma ganharam só a cor
da moldura neste ciclo — o conteúdo deles continua em texto simples.
Dar a cada um o próprio tratamento de conteúdo (badge por tag no
calendário, célula de select como badge na tabela, barra proporcional
no cronograma) é trabalho de ciclo próprio, na mesma esteira desta.

Cor por VARIANTE do callout (info/sucesso/atenção/erro/dica) também
ficou de fora: a moldura do callout tem uma cor fixa (`warning`) por
enquanto — dar cor por variante exigiria fazer a variante viajar até o
desenho da moldura, que hoje só conhece o NOME do embed, não o
conteúdo dele. Fica anotado pro próximo corte.

## Critérios de aceite

- [x] Os nove embeds sem o fluxo têm cor de moldura própria, distinta
      entre si e vinda do CSS (não hardcoded)
- [x] Abrir e fechar um embed não troca a cor do rótulo
- [x] O cartão do kanban vira retângulo preenchido, esticado até a
      borda da coluna
- [x] A miniatura da galeria vira selo preenchido, do tamanho da
      legenda
- [x] Nenhum dos dois mostra mais o nome da parte como prefixo
      (`card Card A` virou `Card A`)
- [x] `variant: primary` nas ações produz uma cor diferente da dos
      outros botões
- [x] A suíte de core e de TUI passam sem quebrar nenhum teste
      existente

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
```
