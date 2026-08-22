---
title: Ciclo 160 — Painel de controle do agent-os
type: ciclo
ciclo: "160"
status: concluida
date: 2026-08-19
prioridade: alta
depende_de: ["151", "154", "155", "156"]
tags:
- ciclo
---

# Ciclo 160 — Painel de controle do agent-os

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Painel de controle do agent-os

## Objetivo

Último ciclo da série. Monta com os embeds novos a interface que o
esquema de agent-os nunca teve: uma página inicial que mostra o estado
real do vault e opera o fluxo do guia sem ninguém tocar em markdown
cru. É também o teste de integração de verdade dos ciclos 151-156 —
com conteúdo real, não fixture.

## Critérios de aceite

- [x] `VaultAnotadinho/pages/produto/painel.md` (novo, `type: landing`,
      definida como página de início) contendo:
      - callout de orientação, linkando o [[Guia do Agent OS]]
      - `actions` com "Nova spec", "Nova decisão", "Novo padrão",
        "Sessão de hoje" (cada um apontando pro template e pasta
        corretos do esquema)
      - as duas listas de spec (`in-progress` e `backlog` por
        prioridade) como `query` EMPILHADAS, não dentro de `columns`:
        embed dentro de embed não existe (não-objetivo declarado no
        ciclo 152, a segmentação só roda no nível da página). O
        `columns` do painel ficou com a referência rápida em texto
      - `query` de decisões recentes, view `cards`
      - `timeline` em `source: vault` mostrando as specs com data
- [x] `guia-agent-os.md` ganha uma seção "Painel" descrevendo a página
      e o que cada bloco faz, e o fluxo recomendado passa a começar
      pelo painel
- [x] `docs/design-system.md` documenta os componentes criados em
      151-156 (classes BEM, variantes, tokens usados)
- [x] `README.md` do repo menciona os tipos de embed disponíveis
- [x] Validação de ponta a ponta ao vivo (MCP `tauri`): clicar "Nova
      spec" cria a página pelo template → ela aparece sozinha na query
      de backlog → mudar `status` pra `in-progress` pelo painel de
      propriedades (ou pelo `anotadinho-cli set-property`) a move pra
      outra lista → a spec com data aparece
      na timeline → nada disso exigiu editar markdown na mão
- [x] O mesmo recorte visto no painel sai igual no terminal via
      `anotadinho-cli query --from-embed pages/produto/painel.md <idx>`

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Trocar o [[Roadmap]] (kanban manual) pelo painel — os dois convivem;
  o roadmap é ordenação intencional, a query é estado derivado
- Onboarding/tour da interface
- Criar o painel automaticamente ao abrir um vault vazio — este ciclo
  entrega a página no vault de exemplo, não um gerador

## Notas

`cargo test --workspace`: 255. `trunk build` e `cargo build
--manifest-path src-tauri/Cargo.toml`: OK.

"Definida como início" é estado LOCAL do usuário (`localStorage`,
chave `anotadinho.home_page::<vault>` — ciclos 089/109), não conteúdo
do vault: quem clonar o vault precisa marcar de novo pelo menu "⋯" →
"Definir como início". Marcado ao vivo nesta validação.

Validação de ponta a ponta (MCP `tauri` + CLI), na ordem do fluxo do
guia:
1. "Nova spec" no painel → pediu o título e criou
   `pages/specs/fluxo-de-ponta-a-ponta.md` a partir do template, com
   `{{title}}`/`{{date}}` resolvidos;
2. a spec apareceu SOZINHA na lista "Fila" (que foi de 1 pra 2 páginas)
   — nada foi movido à mão;
3. `anotadinho-cli set-property ... status in-progress` no terminal →
   ao reabrir o painel ela tinha migrado pra "Em andamento" (1 página)
   e sumido da fila;
4. apareceu também na `timeline` em modo vault (pega o `date` do
   template);
5. `anotadinho-cli query --from-embed pages/produto/painel.md:2`
   devolveu exatamente a mesma linha que a página mostra.

A spec de teste foi removida do vault no fim.

Se algum embed da série não sobreviver ao uso real aqui, o conserto
vira task nova (regra de isolamento do `cycles/README.md`), não um
remendo dentro deste ciclo.

## Resultado

# Ciclo 160 - done

# Fecha a série 148-160

## Resumo

`pages/produto/painel.md` (`type: landing`) monta com os embeds novos a
interface que o esquema de agent-os nunca teve: callout de orientação,
botões que criam spec/decisão/padrão/sessão na pasta certa, listas
vivas de specs por status, decisões recentes em cartões, cronograma do
vault e referência rápida em duas colunas.

Também é o teste de integração de verdade dos ciclos 151-156: conteúdo
real, fluxo real, sem fixture.

## Arquivos criados/modificados

- `VaultAnotadinho/pages/produto/painel.md` (novo)
- `VaultAnotadinho/pages/produto/guia-agent-os.md` — seção "O Painel",
  mapa do vault e fluxo recomendado começando por ele
- `docs/design-system.md` — seção de embeds inline (os 9 tipos e as
  convenções que valem pros nove)
- `README.md` — tabela de embeds e estrutura de crates atualizada

## Testes adicionados

- Nenhum automático: o ciclo é conteúdo + documentação. A validação foi
  o fluxo de ponta a ponta ao vivo (ver task).

## Problemas encontrados

- As duas listas de spec não puderam ir lado a lado dentro do
  `columns`: embed dentro de embed não existe (não-objetivo declarado
  no ciclo 152 — a segmentação só roda no nível da página). Ficaram
  empilhadas, e o `columns` levou a referência rápida em texto.
- "Definida como início" é `localStorage` por vault, não conteúdo do
  vault — quem clonar precisa marcar de novo.

## Notas para próximos ciclos

- Série 148-160 fechada: 3 ciclos de base, 6 embeds novos, 2 de CLI,
  1 de polimento e 1 de painel.
- Fica pendente o que nasceu no caminho: 161 (Escape em modal
  desseleciona a página), 162 (frontmatter reserializado ganha nulos),
  163 (modal de configuração de botão do embed de ações).
