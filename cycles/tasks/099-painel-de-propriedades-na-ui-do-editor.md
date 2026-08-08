---
id: "099"
titulo: "Painel de propriedades na UI do editor"
status: done
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: ["098"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Painel de propriedades na UI do editor

## Objetivo

Segundo ciclo do tema "agent-os readiness". Com `Frontmatter.extra`
preservando propriedades arbitrárias (ciclo 098), esta parte dá uma UI
pra ver/adicionar/editar/remover essas propriedades sem precisar editar
YAML cru na mão — painel colapsável no topo do editor, acima do corpo
da página (mesmo espírito do painel de Backlinks, ciclo 088, que já
fica no fim).

## Critérios de aceite

- [x] `ui/src/components/properties_panel.rs` novo: lista cada
      propriedade (fixas — title/tags/type — E as de `extra`) como uma
      linha `chave: valor` editável; tags renderiza como chips
      (reaproveita o padrão visual já usado no kanban/calendário)
- [x] Botão "+ propriedade" adiciona um par chave-valor novo em
      `extra`; cada linha de `extra` tem um botão de remover
- [x] Editar qualquer campo atualiza o frontmatter e passa pelo mesmo
      caminho de save/autosave já existente (`mark_edited`/`persist`)
- [x] `type:` continua editável só entre os tipos conhecidos
      (md/kanban/calendar/table/tags/assets/landing) — não vira um
      campo de texto livre, pra não quebrar o roteamento de
      `page_view.rs`
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

- Tipos de valor ricos na UI (date picker pra valores que parecem
  data, número com stepper) — v1 edita tudo como texto livre, convertido
  de/para `serde_yaml::Value` como string simples
- Reordenar propriedades por drag — ordem é sempre alfabética
  (`BTreeMap`, já vem assim do ciclo 098)
- Renomear uma chave existente (só editar valor ou remover+adicionar
  nova) — renomear precisaria de lógica extra pra não perder o valor
  durante a operação, fica pra depois se pedirem

## Notas

Reaproveita o mesmo mecanismo de `content_md`/`recompute`/
`mark_edited` já usado pelo resto do editor — o painel de propriedades
edita só a PARTE de frontmatter do `content_md` (via
`MarkdownCodec::split_frontmatter_text`/reconstrução), preservando o
corpo intocado.

Validado ao vivo via MCP `tauri`: editar título/tags/tipo/propriedade
customizada no painel, salvar, e ler o `.md` cru do vault confirma que
tudo persiste corretamente e o corpo da página fica intocado.

Mudar `type:` no painel não troca o dispatch de `page_view.rs` até
salvar+reabrir a página — o dispatch é decidido no fetch da `Page`, não
reage ao `content_md` local. Aceitável pro v1 (já coberto pelos
Não-objetivos).

Bug pré-existente encontrado durante a validação, **não relacionado a
este ciclo**: salvar qualquer página que tenha uma tabela Markdown
(`| a | b |`) achata a tabela pra texto corrido sem os `|`, mesmo sem
tocar no painel de propriedades — reproduzido com uma edição de corpo
comum (sem envolver frontmatter). É um bug de round-trip DOM→Markdown
em `recompute_markdown_from_dom`/`html_to_md.rs` especificamente pra
`<table>`. Vale um ciclo de correção futuro (fora do escopo 098-108
planejado).

**Corrigido no ciclo 111** — ver `cycles/tasks/111-*.md`.
