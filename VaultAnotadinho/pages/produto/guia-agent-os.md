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
│   ├── painel.md          type: landing — o painel de controle (comece aqui)
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

## O Painel

[[Painel]] (`type: landing`) é o ponto de entrada operável do esquema —
uma página comum, montada com embeds inline, que dá pra abrir como
página inicial do vault (menu "⋯" → "Definir como início"):

| Bloco | O que faz |
|---|---|
| `callout` | orientação fixa, linkando este guia |
| `actions` | botões que criam spec/decisão/padrão/sessão **já na pasta certa**, a partir dos templates |
| `query` "Em andamento" | specs com `status: in-progress` |
| `query` "Fila" | specs em `backlog`, ordenadas por `priority` |
| `query` "Decisões" | decisões recentes, em cartões |
| `timeline` | specs com `date::`/`start::` numa linha do tempo (modo vault, leitura) |
| `columns` | referência rápida de status e do comando de terminal |

As listas são **derivadas**: mudou o `status` no frontmatter (pelo
painel de propriedades, pelo botão, ou por
`anotadinho-cli set-property`), a spec troca de bloco sozinha na
próxima abertura. Não existe nada pra manter na mão aqui — diferente do
[[Roadmap]], que é ordenação INTENCIONAL e continua sendo movido por
quem decide a prioridade.

Um agente headless roda exatamente as mesmas consultas:

```bash
anotadinho-cli --vault VaultAnotadinho query --from-embed pages/produto/painel.md:2
```

## Fluxo recomendado (humano ou agente)

0. Abrir o [[Painel]] — o que está em andamento e o que está na fila
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

### Consultas (ciclo 158)

`query` é o MESMO motor do embed `{{ type: "query" }}` — o recorte que
você vê no terminal é literalmente o que o humano vê na página.

```bash
# specs que ainda não estão prontas, por prioridade
anotadinho-cli --vault VaultAnotadinho query \
  --from pages/specs --where 'status!=done' --sort priority --field status

# combina condições (AND): =, !=, ~ (contém), ? (existe), > e <
anotadinho-cli --vault VaultAnotadinho query --tag spec --where 'priority?' --json

# roda a consulta que já está declarada num embed de uma página
anotadinho-cli --vault VaultAnotadinho query --from-embed pages/produto/painel.md:0
```

> `!=` casa também com página que NÃO TEM o campo — uma spec sem
> `status` é justamente trabalho não classificado, e sumir com ela do
> recorte seria o pior erro possível aqui.

### Embeds (ciclo 157)

Mexer num board/tabela/calendário sem reescrever o `.md` na mão (e sem
montar YAML por concatenação, que já corrompeu arquivo no ciclo 064):

```bash
# o que tem de embed nesta página?
anotadinho-cli --vault VaultAnotadinho embed list pages/produto/painel.md

# lê o conteúdo do embed 0 (YAML; tabela markdown no tipo `table`)
anotadinho-cli --vault VaultAnotadinho embed get pages/produto/roadmap.md 0

# grava de volta (do stdin ou de --file); passa pelo parser antes de escrever
anotadinho-cli --vault VaultAnotadinho embed set pages/produto/painel.md 0 --file novo.yaml

# atalhos
anotadinho-cli --vault VaultAnotadinho embed add-card pages/x.md 0 --column Todo --title "Nova tarefa"
anotadinho-cli --vault VaultAnotadinho embed add-row pages/x.md 1 --values "API, done, alta"
anotadinho-cli --vault VaultAnotadinho embed add-event pages/x.md 2 --date 2026-09-01 --title "Revisão"
```

Com o app aberto, a mudança aparece na hora — o watcher recarrega a
página. Uma escrita re-serializa todos os embeds daquela página (não só
o alterado), então o primeiro `git diff` costuma vir maior do que a
mudança em si.

O binário sai empacotado junto com a GUI a partir de `./scripts/build.sh`
(ciclo 114) — não precisa de build separado pra ter os dois.
