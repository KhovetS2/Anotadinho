---
title: Guia do Agent OS
tags:
- produto
- guia
---
# Guia do Agent OS

> Página fixa — o ponto de entrada de tudo. Inspirada no
> [Agent OS](https://buildermethods.com/agent-os) (Brian Casel):
> 3 camadas — **produto** (por que construir), **specs** (o que
> construir a seguir) e **padrões** (como construir) — adaptadas ao
> que o Anotadinho já suporta nativamente: pastas, templates com
> `{{title}}`/`{{date}}`, frontmatter customizável (painel de
> propriedades), tags, kanban, e o CLI headless (`anotadinho-cli`).

## Mapa do vault

```
pages/
├── produto/              ← páginas FIXAS, uma de cada (não usam template)
│   ├── missao.md          "por que" o produto existe
│   ├── roadmap.md          type: kanban — backlog/todo/doing/done
│   ├── stack-tecnico.md    tecnologias escolhidas e por quê
│   ├── grafo.md            type: graph — conexões via [[wikilinks]]
│   └── guia-agent-os.md    esta página
├── specs/                 ← uma página por feature (via template "spec")
├── decisoes/               ← uma página por decisão (via template "decisão")
└── padroes/                ← uma página por domínio de padrão de código
                              (via template "padrão de código")

journals/                  ← log diário nativo do Anotadinho (botão "Hoje")
templates/                  ← os 5 templates usados por tudo isso
```

## As 3 camadas

**Produto** (`pages/produto/`) — por que construir. Poucas páginas
fixas, editadas direto (não via template, porque só existe uma de
cada). Toda spec deveria conseguir se ligar de volta à
[[Missão]] ou ao [[Roadmap]].

> Nota técnica sobre o [[Roadmap]] (`type: kanban`): o parser do board
> lê o arquivo `.md` inteiro (frontmatter incluído) procurando linhas
> `- `, sem separar frontmatter do corpo — uma lista YAML bloco no
> frontmatter (ex: `tags:\n- produto`) vira um card falso. Use sempre
> `tags: [produto]` (flow style) no frontmatter de páginas
> `type: kanban`.

**Specs** (`pages/specs/`) — o que construir a seguir. Uma página por
feature, criada via template "spec". É o documento mais completo do
esquema — contexto, objetivo, escopo dentro/fora, requisitos,
design técnico, plano de tarefas, critérios de aceite testáveis,
comandos de validação, riscos, não-objetivos. Ver
[[Exemplo — Exportar nota em PDF]] preenchida de ponta a ponta.

**Padrões** (`pages/padroes/`) — como construir. Uma página por
domínio (API, testes, nomenclatura, frontend...), criada via template
"padrão de código". Ver [[Nomenclatura]] como exemplo.

O [[Grafo do Vault]] (`type: graph`) mostra visualmente essas conexões
— todas as páginas como nós, wikilinks como arestas.

Decisões (`pages/decisoes/`) atravessam as 3 camadas — registram
POR QUE uma escolha técnica ou de produto foi feita, geralmente
nascendo de dentro de uma spec (campo `related_spec`). Ver
[[Exemplo — Não usar lib de geração de PDF no backend]].

## Convenção de frontmatter

| Campo | Onde | Valores |
|---|---|---|
| `status` (spec) | specs | `backlog` → `in-progress` → `in-review` → `done` (ou `blocked`) |
| `status` (decisão) | decisões | `proposta` → `aceita` / `rejeitada` / `substituída` |
| `priority` | specs | `alta` / `media` / `baixa` |
| `dominio` | padrões | livre (`api`, `frontend`, `testes`, `nomenclatura`...) |
| `tags` | todas | `spec`/`decisao`/`padrao`/`sessao` + tags livres |
| `depends_on` | specs | lista de paths de outras specs |
| `related_decision` / `related_spec` | specs / decisões | path da página relacionada |

Todos esses campos são editáveis pelo painel de propriedades (botão
"⋯" no editor) ou direto no `.md` — o painel não perde propriedades
customizadas (`extra`, ciclo 098).

## Fluxo recomendado (humano ou agente)

1. Checar [[Roadmap]] — qual spec está em `todo`/próxima da fila
2. Abrir a spec em `pages/specs/`, ler contexto + plano de tarefas
3. Se precisar de uma decisão de arquitetura no meio do caminho,
   registrar em `pages/decisoes/` (template "decisão"), linkando de
   volta via `related_spec`
4. Trabalhar a spec: marcar tarefas concluídas, mudar `status` pra
   `in-progress` e depois `done`
5. Ao fim da sessão, registrar um resumo — página do `journals/`
   (botão "Hoje") pra log rápido, ou template "sessão de trabalho" se
   quiser algo mais estruturado e ligado a uma spec específica
6. Mover o card da spec no [[Roadmap]] pra `done`

## Operando via CLI (`anotadinho-cli`)

Pra um agente headless (sem a janela do Tauri aberta):

```bash
# lista specs em backlog (filtro nativo, ciclo 115)
anotadinho-cli --vault VaultAnotadinho list-pages --folder pages/specs --status backlog

# lê uma spec inteira (frontmatter + corpo)
anotadinho-cli --vault VaultAnotadinho read pages/specs/exemplo-exportar-nota-em-pdf.md

# muda o status de uma spec sem editar o .md na mão (ciclo 116)
anotadinho-cli --vault VaultAnotadinho set-property pages/specs/minha-spec.md status in-progress

# busca full-text em specs/decisões/padrões
anotadinho-cli --vault VaultAnotadinho search "termo"

# dump de todas as specs, pra dar de contexto pra um LLM
anotadinho-cli --vault VaultAnotadinho export --folder pages/specs

# cria uma spec nova a partir do template, com {{title}}/{{date}} resolvidos
anotadinho-cli --vault VaultAnotadinho new-from-template templates/spec.md "Minha feature nova"
```

O binário sai empacotado junto com a GUI a partir de `./scripts/build.sh`
(ciclo 114) — não precisa de build separado pra ter os dois.
