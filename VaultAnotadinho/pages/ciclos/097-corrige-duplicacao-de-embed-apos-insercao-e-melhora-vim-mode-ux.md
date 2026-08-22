---
title: Ciclo 097 — Corrige duplicacao de embed apos insercao e melhora vim mode UX
type: ciclo
ciclo: "097"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 097 — Corrige duplicacao de embed apos insercao e melhora vim mode UX

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Corrige duplicação de embed após inserção + melhora UX do vim mode

## Objetivo

Bug report do usuário (com screenshot): ao montar uma landing page,
inserir uma tabela logo depois de um calendário (com vim mode ativado)
deixava texto cru de markdown duplicado ao lado do embed de verdade.
Também pedido: mostrar o modo do vim (Normal/Insert) na barra de baixo,
aumentar o padding interno do editor, e destacar mais o marcador de
início de linha (bullet).

## Critérios de aceite

- [x] `ui/src/components/editor.rs`: `key` de cada segmento na lista
      `{ for segments.iter().enumerate().map(...) }` passa a incluir o
      TIPO do segmento (`md`/`embed`), não só a posição — o caso real
      era um segmento Markdown virar Embed NO MESMO ÍNDICE com a
      CONTAGEM total igual antes/depois (linha vazia + `/tabela` colada
      nela vira só o embed, sem sobrar markdown), que o key antigo
      (só posição) não detectava — Yew reaproveitava o `<div>` e só
      ANEXAVA os filhos novos depois do marcador de inserção que já
      estava lá (imperativo, invisível pro Yew), em vez de desmontar/
      remontar de verdade
- [x] Indicador `-- NORMAL --`/`-- INSERT --` na barra de status do
      editor, só quando vim mode está ativado
- [x] Padding do editor (`--sp-4` → `--sp-6`) nos dois modos (com e sem
      embeds)
- [x] `::marker` dos bullets de lista fica maior, em negrito e na cor
      de destaque (era a cor de texto padrão, discreto demais)
- [x] Bug relacionado encontrado e corrigido: quando o popup do menu
      `/`/wikilink estava aberto em modo Insert do vim, `Escape`
      caía no handler do vim (Insert→Normal) ANTES de chegar no
      handler do próprio popup — o popup ficava preso aberto (visual),
      o texto cru "/consulta" nunca era apagado, e Enter/setas viravam
      motion do vim em vez de navegar o popup. Reordenado: popups
      SEMPRE têm prioridade sobre a interceptação do vim mode
- [x] `u` (undo) do vim mode passa a chamar o MESMO `do_undo` do
      `Ctrl+Z` em vez de `execCommand("undo")` nativo — o undo nativo
      do contenteditable operava fora do controle de `content_md`/
      `undo_stack`, risco de desincronizar os dois
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Migrar TODOS os `key`s da árvore de segmentos pra um esquema de ID
  estável e independente de posição (ex: UUID por segmento persistido)
  — o key posição+tipo resolve o caso real reportado sem precisar de
  uma refatoração maior de como segmentos são identificados
- Voltar a permitir `execCommand` genérico em outros pontos do editor —
  só o `u` do vim mode foi trocado, o resto do editor já usa
  `Range`/`insert_element_at_cursor` desde os ciclos 079/084

## Notas

**Diagnóstico** (o mais trabalhoso do ciclo): a suspeita inicial era que
o bug fosse ESPECÍFICO do vim mode (o usuário mencionou "modo vim
ativado" no relato) — a causa raiz real são DUAS coisas relacionadas
mas distintas:

1. Um bug de vim mode de verdade: `Escape` com o popup do menu `/`
   aberto em modo Insert caía no handler errado (comportamento
   confuso, mas não é o que gera o texto duplicado do screenshot).
2. Um bug GERAL de reconciliação do Yew (não específico de vim mode,
   só mais fácil de disparar testando fluxos incomuns como vim mode) —
   inserir um embed numa posição onde ANTES só havia markdown vazio,
   sem mudar a CONTAGEM total de segmentos, fazia o Yew reaproveitar o
   `<div>` errado. Confirmado isolando: reproduzi o MESMO bug sem vim
   mode nenhum ativado, só inserindo calendário → "+" linha abaixo →
   tabela em sequência rápida.

Processo de diagnóstico: comparar `innerHTML` da árvore de segmentos
logo após a inserção (parecia limpo — só verifiquei os filhos de
PRIMEIRO NÍVEL) vs. depois de salvar+recarregar (sempre limpo, prova
que `content_md`/o arquivo salvo sempre estavam corretos — o bug era
100% de DOM ao vivo, nunca de dados) — aí sim procurando
`div[data-embed-insert]` remanescente na árvore inteira, achando ele
como PRIMEIRO FILHO do `.embed-hover-wrapper` da tabela (confirma:
Yew reaproveitou aquele `<div>`, só ANEXOU os filhos novos depois do
que já estava lá).

Primeira tentativa de fix (key = `segments.len()-i`) NÃO resolveu —
contagem de segmentos ficou em 3 antes E depois da inserção (a linha
vazia onde o usuário digitou `/tabela` foi consumida inteira pelo
embed, sem sobrar segmento de markdown separado), então o key não
mudava. Fix de verdade: incluir o TIPO do segmento (`md` vs `embed`)
no key, não só contagem+posição.

Achado de metodologia de teste (reforça notas de ciclos anteriores):
`document.body.innerText.includes(...)` é mais confiável que o
screenshot via html2canvas (que já tinha se mostrado não-fiel ao DOM
real antes nesta sessão) pra confirmar presença/ausência de texto.

## Resultado

# Ciclo 097 - done

## Resumo

Bug report do usuário: texto de tabela cru duplicado ao lado do embed
de verdade ao inserir uma tabela depois de um calendário. Causa raiz:
bug de reconciliação do Yew (key de segmento não capturava a troca
Markdown→Embed no mesmo índice) + um bug real, relacionado mas
distinto, de prioridade de teclado do vim mode com popups abertos.
Também: indicador de modo do vim na status bar, mais padding no
editor, bullets mais visíveis.

## Arquivos criados/modificados

- `ui/src/components/editor.rs` — key por tipo+posição+contagem na
  lista de segmentos; popups (`/`/wikilink) com prioridade sobre
  interceptação do vim mode; `u` do vim mode usa `do_undo` em vez de
  `execCommand`; indicador de modo na status bar
- `ui/src/styles/main.css` — padding do editor, `::marker` dos
  bullets, `.editor__vim-mode*`, `.editor__statusbar` ajustado

## Testes

Sem testes novos (bug de reconciliação DOM/timing, validado ao vivo
com reprodução exata do cenário reportado). `cargo test --workspace`:
61. `cd ui && cargo test --lib`: 66. Total 127.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Reproduzido o bug exato (calendário → "+" linha → `/tabela` → Tabela
de Tarefas) confirmando texto duplicado ANTES do fix; confirmado limpo
(0 `div[data-embed-insert]` remanescentes) DEPOIS do fix, tanto
imediatamente quanto após salvar+recarregar. Indicador de modo do vim
confirmado alternando NORMAL/INSERT; padding confirmado via
`getComputedStyle` (24px). Detalhes no arquivo de task.

## Notas

Fix de reconciliação (`key` por tipo de segmento) é potencialmente
relevante pra QUALQUER inserção de embed via slash em posições onde só
havia markdown vazio antes, não só o caso tabela-depois-de-calendário
testado — mas esse foi o caso reproduzido e confirmado.
