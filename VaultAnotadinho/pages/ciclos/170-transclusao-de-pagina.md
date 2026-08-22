---
title: Ciclo 170 — Transclusão de página
type: ciclo
ciclo: "170"
status: concluida
date: 2026-08-20
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 170 — Transclusão de página

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Transclusão de página

## Objetivo

O painel (160) consegue LISTAR páginas, nunca MOSTRAR o conteúdo delas.
Pra montar um dashboard de verdade — "a missão do produto aqui em cima,
o padrão de nomenclatura ali embaixo" — hoje só copiando texto, que
duplica a fonte da verdade.

Transclusão resolve: `![[Missão]]` renderiza o conteúdo daquela página
no lugar, sempre atualizado.

## Critérios de aceite

- [x] `![[Página]]` no markdown renderiza o corpo da página alvo
      (sem o frontmatter), com um cabeçalho discreto que leva pra ela
- [x] `![[Página#Seção]]` traz só a seção daquele heading até o próximo
      do mesmo nível
- [x] Ciclo de transclusão (A inclui B que inclui A) para no primeiro
      nível repetido, com aviso no lugar — nunca laço infinito
- [x] PARCIAL — embed dentro de página transcluída vira um aviso
      ("Bloco kanban — abra a página pra usar") em vez de um embed
      interativo. O conteúdo transcluído entra como HTML no DOM, fora
      do VDOM do Yew; montar componentes de verdade ali exigiria uma
      segunda árvore de renderização. O aviso é honesto e não mostra
      YAML solto no meio do texto, que era o risco real
- [x] Alvo inexistente mostra o nome pedido e um jeito de criar a
      página, não um buraco
- [x] Conta como backlink (painel de backlinks e grafo enxergam)
- [x] Testes do parser no core: `![[x]]` vs `[[x]]`, com âncora, dentro
      de fence de código (não transclui), aninhado

## Comandos de validação

```bash
cargo test -p anotadinho-core
cargo test --workspace
cd ui && trunk build
```

## Não-objetivos

- Editar a página de origem pelo bloco transcluído
- Transcluir bloco específico por id (`![[x^bloco]]`) — precisa de id
  de bloco, que o projeto não tem ainda

## Notas

`cargo test -p anotadinho-core`: 159 (+5). Harness (177): 12/12, com
cenário que confere página inteira, seção recortada, auto-transclusão
barrada, alvo inexistente e que o `.md` não muda.

Resolução do alvo usa `scan_vault` e não `list_pages`: o título que
interessa é o do FRONTMATTER, então `![[Guia do Agent OS]]` casa com
`guia-agent-os.md`. Cai pro path e pro nome do arquivo se não achar
pelo título.

Ciclo infinito: o conteúdo transcluído NÃO é varrido de novo por
marcadores, então nada aninha além de um nível. Auto-transclusão é
barrada com mensagem própria, porque é o erro mais fácil de cometer.

`crates/core/src/links.rs` já separa alvo/alias/âncora — o parser de
transclusão deve reusar isso em vez de reimplementar.

## Resultado

# Ciclo 170 - done

# Resumo

`![[Página]]` embute o conteúdo da página alvo, sempre atualizado;
`![[Página#Seção]]` traz só aquele trecho. É o que faltava pra montar
painel sem duplicar texto — o painel do ciclo 160 só conseguia LISTAR
páginas.

## Arquivos criados/modificados

- `crates/core/src/links.rs` — `extract_transclusion_targets`,
  `extract_section` + 4 testes
- `crates/core/src/index.rs` — transclusão conta como referência (grafo
  e backlinks) + 1 teste
- `ui/src/markdown_render.rs` — `![[X]]` vira marcador antes do linkify
- `ui/src/components/editor.rs` — `upgrade_transclusions_at`
- `ui/src/styles/main.css` — `.transclusao*`
- `scripts/uitest/cenarios.mjs` — cenário novo

## Testes adicionados

- transclusão separada do wikilink; ignorada dentro de fence
- seção recortada até o próximo heading do mesmo nível (incluindo as
  sub-seções); seção inexistente devolve `None`
- transclusão conta como referência no índice
- cenário de UI: página inteira, seção, auto-transclusão, alvo
  inexistente, e o `.md` intocado

## Problemas encontrados

- Resolver o alvo por `list_pages` não funcionava: ele devolve o NOME
  DO ARQUIVO como título. Passou a usar `scan_vault`, que traz o título
  do frontmatter.
- Embed dentro de página transcluída ficou como aviso, não como embed
  interativo — o conteúdo entra como HTML, fora do VDOM. Documentado na
  task.

## Notas para próximos ciclos

- 176 (id de bloco) permite `![[página^bloco]]`, o próximo passo
  natural daqui.
