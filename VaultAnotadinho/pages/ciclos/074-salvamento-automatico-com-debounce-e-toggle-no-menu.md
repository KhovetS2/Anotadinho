---
title: Ciclo 074 — Salvamento automatico com debounce e toggle no menu
type: ciclo
ciclo: "074"
status: concluida
date: 2026-08-07
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 074 — Salvamento automatico com debounce e toggle no menu

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Salvamento automático com debounce e toggle no menu

## Objetivo

Usuário pediu: salvamento automático alguns segundos depois de parar de
digitar, opção pra ativar/desativar no menu principal, e — o problema de
fundo — parar de perder o estado da página ao trocar de página sem salvar
antes.

## Critérios de aceite

- [x] Toggle "Salvamento automático" no menu ⚙ (`HeaderBar`), persistido
      em `localStorage` (`anotadinho.autosave_enabled`, padrão ativado)
- [x] Com o toggle desligado, editar não agenda mais o save automático
      de 3s — só marca "não salvo", precisa clicar em "Salvar"
- [x] Trocar de página com edições pendentes salva automaticamente antes
      da troca, **independente do toggle** — essa parte é uma rede de
      segurança contra perda de dado, não a conveniência do autosave
- [x] Testes de regressão (via MCP, ver Notas — não dá pra testar isso
      com `cargo test` puro, depende de DOM real)

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum indicador visual de "salvando..." diferente do que já existia
- Resolver o `#[derive]`/lint de `cargo clippy` nas outras partes do
  arquivo — fora de escopo

## Notas

**Descoberta técnica central**: o autosave debounced (3s) já existia
antes deste ciclo (`trigger_debounced_save`), só faltavam o toggle e a
proteção contra perda ao trocar de página. O ponto difícil foi a
proteção: o efeito que observa `props.page` (`Effect 1` do `Editor`) só
recria seus closures quando a PÁGINA muda — lendo `*edited`/`*content_md`
(ambos `use_state`) de dentro do cleanup desse efeito sempre devolvia o
valor de QUANDO O EFEITO FOI CRIADO (`edited=false`, `content_md` vazio),
nunca o estado atual, porque `UseStateHandle` é um snapshot por
renderização — um clone capturado num efeito que só roda de novo quando
`page` muda fica congelado, mesmo que `.set()` seja chamado depois por
OUTRO clone do mesmo handle (mesma classe de bug já encontrada e corrigida
no resize do calendário, ciclo 071).

**Solução**: `edited_ref`/`pending_flush_ref`, dois `use_mut_ref`
(`Rc<RefCell<_>>` — a MESMA instância em toda renderização, ao contrário
de `use_state`). Atualizados a cada `oninput` (dentro de
`trigger_debounced_save`, que roda a cada tecla, então sempre lê valores
frescos de `content_md`/`editor_ref`/`segment_refs` — diferente do efeito
de página, essa closure É recriada a cada renderização). O cleanup do
`Effect 1` lê esses refs (sempre atuais) e, se havia edição pendente,
dispara `api::write_page` pra página que está sendo deixada, direto — sem
depender de handles `use_state` potencialmente congelados.

**Bug real encontrado de bônus**: extrair a lógica de recomputar markdown
do DOM (`do_save` → `recompute_markdown_from_dom`) revelou que o branch
de página SEM embeds nunca recolocava o frontmatter (`fm`) na frente do
corpo recomputado — só o branch COM embeds fazia isso certo. Ou seja,
**qualquer save de uma página com frontmatter e sem embeds já perdia o
frontmatter inteiro antes deste ciclo**, não é bug introduzido agora.
Corrigido no mesmo commit.

Validado ao vivo via MCP `tauri` na página `teste` (frontmatter
`title: teste` + 1 bullet): digitar texto e trocar de página IMEDIATAMENTE
(sem esperar os 3s) — reabrir a página mostrou o texto novo E o
frontmatter intacto (antes do fix de frontmatter, `title:: teste` sumia
do arquivo). Repetido com o toggle de autosave DESLIGADO: esperei 4s sem
o conteúdo ser salvo (confirmando que o timer de 3s não disparou), depois
troquei de página e confirmei que o flush de segurança salvou mesmo assim.
Nenhuma edição de teste vazou pro vault — os dois testes na página
`teste.md` foram revertidos com `git checkout` depois de confirmados.

## Resultado

# Ciclo 074 - done

## Resumo

Toggle de salvamento automático no menu ⚙ + rede de segurança que salva
a página antes de trocar pra outra, mesmo com o toggle desligado —
corrige a perda de texto ao navegar sem salvar. De bônus, corrige um bug
pré-existente de perda de frontmatter em qualquer save de página sem
embeds.

## Arquivos criados/modificados

- `ui/src/state.rs` — `save_autosave_enabled`/`load_autosave_enabled`
  (localStorage, padrão ativado)
- `ui/src/app.rs` — estado `autosave_enabled` + callback de toggle,
  repassado pra `HeaderBar` e `PageView`
- `ui/src/components/header_bar.rs` — prop `autosave_enabled` +
  `on_toggle_autosave`, item de menu novo
- `ui/src/components/page_view.rs` — repassa `autosave_enabled` pro
  `Editor`
- `ui/src/components/editor.rs` — `edited_ref`/`pending_flush_ref`
  (`use_mut_ref`, não `use_state` — evita o handle congelado do efeito de
  página), flush no cleanup do efeito que observa `props.page`,
  `recompute_markdown_from_dom` extraída de `do_save` e corrigida pra
  preservar frontmatter no branch sem embeds, debounce de 3s gateado por
  `props.autosave_enabled`

## Testes

`cargo test --lib`: 52 passaram (sem testes novos — lógica depende de DOM
real via `NodeRef`, sem harness de wasm-bindgen-test no projeto; validado
via MCP ao vivo).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Digitar na página `teste` e trocar de página imediatamente (bem antes dos
3s) — reabrir mostrou o texto novo salvo E o frontmatter (`title: teste`)
intacto. Repetido com o toggle desligado: esperei 4s confirmando que o
timer de 3s não disparava (arquivo no disco continuou com o conteúdo
anterior), troquei de página e confirmei que o flush de segurança salvou
mesmo assim. `git diff --stat VaultAnotadinho/` vazio depois de reverter
as duas edições de teste na `teste.md`.

## Notas

Bug de frontmatter (achado durante a extração de `do_save`) era
pré-existente — qualquer save manual de uma página com frontmatter e sem
embeds já perdia o frontmatter antes deste ciclo. Corrigido no mesmo
commit por estar diretamente no caminho do código mexido.
