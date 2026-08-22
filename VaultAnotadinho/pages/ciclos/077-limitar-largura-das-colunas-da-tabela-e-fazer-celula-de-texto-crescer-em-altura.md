---
title: Ciclo 077 — Limitar largura das colunas da tabela e fazer celula de texto crescer em altura
type: ciclo
ciclo: "077"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: ["076"]
tags:
- ciclo
---

# Ciclo 077 — Limitar largura das colunas da tabela e fazer celula de texto crescer em altura

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Limitar largura das colunas da tabela e fazer célula de texto crescer em altura

## Objetivo

Colunas da tabela embed cresciam em largura conforme o conteúdo mais
comprido de qualquer célula (layout `auto` padrão), espremendo as
colunas vizinhas. Trocado pra `table-layout: fixed` (largura igual por
coluna, definida pela linha de cabeçalho, não muda por causa do
conteúdo) e a coluna `Text` (já trocada de `contenteditable` pra
elemento de verdade no ciclo 076) agora usa `<textarea>` com altura
ajustada automaticamente via JS em vez de `<input>` de uma linha só.

## Critérios de aceite

- [x] `.task-table__table { table-layout: fixed; }` — colunas com
      largura igual, sem esticar por causa de conteúdo comprido
- [x] Colunas com largura própria (`.task-table__th--add`/
      `.task-table__td--actions`, 32px) continuam do tamanho certo
- [x] Coluna `Text` vira `<textarea>` (não `<input>`) com altura
      ajustada via JS (`oninput`/`onfocus`) em vez de recortar/rolar
      texto horizontalmente
- [x] `overflow-wrap`/`word-break` nas células pra texto comprido quebrar
      em vez de vazar da coluna

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Redimensionar coluna arrastando a borda (resize manual) — fica pra um
  ciclo futuro se for pedido
- Auto-grow do `<textarea>` ao CARREGAR a página com conteúdo já
  comprido (sem interação) — só ajusta ao focar/digitar; ver Notas

## Notas

Continuação direta do ciclo 076: a troca de `contenteditable` pra
`<input>` resolveu a duplicação de texto, mas um `<input>` de uma linha
só não consegue "crescer em altura" (é fisicamente incapaz de mostrar
mais de uma linha). Trocado por `<textarea>`, que preserva a mesma
propriedade que resolveu o bug de duplicação (valor é propriedade do
elemento, não filhos de DOM reconciliados pelo virtual DOM) e ainda
suporta múltiplas linhas.

Auto-grow implementado via JS (`autogrow_textarea`: zera a altura, mede
`scrollHeight`, aplica de volta) chamado em `oninput` (digitando) e
`onfocus` (clicar pra ver/editar uma célula com texto longo já
existente). Não roda automaticamente no MOUNT da página (só quando o
usuário interage com a célula) — para uma tabela com muitas linhas,
rodar uma medição de DOM em toda `<textarea>` a cada carregamento seria
caro; o trade-off é que uma célula com texto longo carregado do disco
aparece com altura de 1 linha (`overflow:hidden`, texto cortado) até o
usuário clicar nela, que aí ajusta a altura certa. Ver "Não-objetivos".

Validado ao vivo via MCP `tauri` na tabela embed de `exemplos-embeds.md`:
larguras de coluna confirmadas iguais (~195px cada, coluna "+" mantendo
32px) antes E depois de digitar um texto longo numa célula — a largura
não mudou. Altura da textarea cresceu de 24px pra 147px
(`scrollHeight`/`clientHeight` batendo), com `white-space: pre-wrap` e
`overflow-wrap: break-word` confirmados via `getComputedStyle`. O
screenshot da ferramenta de teste (via `html2canvas`) não renderiza o
texto quebrado dentro do `<textarea>` corretamente — limitação conhecida
do `html2canvas` com conteúdo interno de campos de formulário, não um bug
do app (confirmado pelos valores computados via JS, que batem
exatamente).

## Resultado

# Ciclo 077 - done

## Resumo

`table-layout: fixed` na tabela embed (colunas com largura igual, não
esticam mais por causa de conteúdo comprido) + coluna `Text` trocada de
`<input>` (ciclo 076) pra `<textarea>` com altura ajustada
automaticamente via JS, pra crescer em altura em vez da coluna crescer
em largura.

## Arquivos criados/modificados

- `ui/src/components/embeds/inline_table.rs` — `textarea_value`,
  `autogrow_textarea`, coluna `Text` vira `<textarea>` com
  `oninput`/`onfocus` ajustando altura
- `ui/src/styles/main.css` — `table-layout: fixed`,
  `overflow-wrap`/`word-break` em th/td, `.task-table__text-input`
  ajustado pra `<textarea>` (resize:none, overflow:hidden, white-space:
  pre-wrap)

## Testes

`cargo test --lib`: 52 passaram (sem testes novos — depende de DOM real
pra medir `scrollHeight`; validado via MCP ao vivo).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Larguras de coluna (~195px cada, "+" mantendo 32px) confirmadas
IDÊNTICAS antes e depois de digitar um texto bem longo numa célula.
Altura da textarea cresceu de 24px pra 147px (`scrollHeight` batendo
com `clientHeight`), CSS de wrap (`white-space: pre-wrap`,
`overflow-wrap: break-word`) confirmado via `getComputedStyle`.

## Notas

Screenshot da ferramenta de teste não mostra o texto quebrado dentro do
`<textarea>` (limitação conhecida do `html2canvas` com conteúdo interno
de campos de formulário) — confirmado que é só limitação da ferramenta,
não bug real, via os valores computados (altura/scrollHeight/CSS batendo
exatamente com o esperado).

Auto-grow só dispara em `oninput`/`onfocus`, não no carregamento da
página — trade-off documentado na task pra evitar medir DOM de toda
`<textarea>` em tabelas grandes a cada load.
