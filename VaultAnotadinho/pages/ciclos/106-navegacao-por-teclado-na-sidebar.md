---
title: Ciclo 106 — Navegacao por teclado na sidebar
type: ciclo
ciclo: "106"
status: concluida
date: 2026-08-08
prioridade: media
depende_de: ["105"]
tags:
- ciclo
---

# Ciclo 106 — Navegacao por teclado na sidebar

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Navegação por teclado na sidebar

## Objetivo

Terceiro ciclo do tema "navegação 100% via teclado". A sidebar (lista
de páginas + árvore de pastas) hoje é 100% mouse — zero navegação por
seta (confirmado na auditoria). Este ciclo implementa a ação "Focar
sidebar" deixada pronta no ciclo 105: setas movem um item destacado
(incluindo entrar/sair de pastas), Enter abre a página destacada,
Escape sai da região e devolve o foco pro editor.

## Critérios de aceite

- [x] `ui/src/components/sidebar.rs`: estado de "item destacado" (índice
      numa lista ACHATADA — pastas expandidas + páginas visíveis, na
      ordem em que aparecem na tela) — mesmo padrão de índice já usado
      pelo menu `/`/paleta de comandos
- [x] `ArrowDown`/`ArrowUp` movem o destaque (scroll-into-view se sair
      da área visível, mesmo padrão do menu `/`); `ArrowRight` numa
      pasta expande (se colapsada) ou entra nela; `ArrowLeft` colapsa
      (se expandida) ou sobe pro pai
- [x] `Enter` abre a página destacada (mesmo `on_page_selected` do
      clique); `Escape` sai da região sidebar, foco volta pro editor
      (ou pro corpo da página se nenhuma estiver aberta)
- [x] Ativar via "Focar sidebar" (`GlobalKeymap`, ciclo 105) muda o
      destaque visualmente pro primeiro item e habilita a navegação por
      seta; clicar com o mouse continua funcionando exatamente como
      antes
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

- Navegação por teclado dentro da caixa de busca da sidebar (já filtra
  em tempo real digitando; setas aqui são só pra navegar a LISTA depois
  de focar a região, não pra dentro do campo de texto)
- Ações de mover/renomear/excluir página só com teclado (continuam só
  por menu/botão) — este ciclo é só navegação/seleção, não gestão
- Journals na mesma navegação por seta da árvore de Pages — v1 cobre só
  a seção Pages (lista + árvore); Journals fica pra depois se pedirem

## Notas

Depende do ciclo 105 (`GlobalKeymap` com a ação "Focar sidebar" já
existindo, mesmo que sem comportamento ainda). A "lista achatada" de
navegação precisa ser recalculada toda vez que uma pasta expande/
colapsa (igual a árvore de pastas já reage a isso via `<details
open>` nativo) — cuidado pra manter o índice destacado coerente quando
a lista muda de tamanho no meio da navegação.

Decisões de implementação:

- Índice por STRING (o `key` de pasta/página — mesmo path já usado por
  `selected_path`), não um `usize` cru — mais robusto a re-renders (não
  depende de posições ficarem em sync entre frames).
- `<details open=true>` sempre-aberto virou CONTROLADO por um novo
  estado `collapsed_folders: HashSet<String>` — precisava ser
  controlável pro `ArrowLeft`/`ArrowRight` funcionarem. Um `ontoggle`
  sincroniza o clique nativo do mouse no `<summary>` de volta pro mesmo
  estado (fonte única, sem os dois competirem).
- Nav funciona tanto na árvore (sem filtro) quanto na lista achatada de
  resultados de busca (com filtro) — mesma função `flatten_nav`,
  chamada com estruturas diferentes conforme o caso.
- **Bug encontrado e corrigido durante a validação**: o `Escape` da
  sidebar não tinha `stop_propagation`, então borbulhava até
  `.app-root`, que tem seu PRÓPRIO caso de Escape (fora do
  `GlobalKeymap`, deseleciona a página aberta) — resultado: sair da
  navegação da sidebar TAMBÉM fechava a página aberta. Corrigido
  adicionando `e.stop_propagation()` no branch de Escape.

Validado ao vivo via MCP `tauri`: "Focar sidebar" (atribuído a Ctrl+J
pro teste) destaca o primeiro item; `ArrowLeft` colapsa uma pasta
aberta; `ArrowRight` expande (1º toque) e entra nela (2º toque);
`Enter` abre a página destacada; `Escape` limpa o destaque E devolve o
foco pro contenteditable do editor, sem fechar a página; clique do
mouse continua funcionando normalmente depois de usar teclado.

## Resultado

# Ciclo 106 - done

## Resumo

Terceiro ciclo do tema "navegação 100% via teclado". A sidebar (árvore
de pastas + lista de páginas) ganha navegação por teclado de verdade:
`ArrowUp`/`ArrowDown` movem um item destacado, `ArrowRight`/`ArrowLeft`
expandem/colapsam pastas (ou entram/saem delas), `Enter` abre a página
destacada, `Escape` sai da região. Ativado pela ação "Focar sidebar" do
`GlobalKeymap` (ciclo 105), que antes não fazia nada.

## Arquivos criados/modificados

- `ui/src/components/sidebar.rs` — `NavItem`/`flatten_nav`,
  `collapsed_folders` (substitui `<details open=true>` sempre-aberto),
  `nav_active`, handler de teclado, `render_tree`/`render_movable_list`
  ganham params de destaque/ref, 4 testes novos de `flatten_nav`
- `ui/src/app.rs` — "Focar sidebar" dispara nonce que a `Sidebar`
  observa; "Focar editor" foca o contenteditable direto
- `ui/Cargo.toml` — feature `HtmlDetailsElement` do web-sys
- `ui/src/styles/main.css` — `.sidebar-item--nav-active`/
  `.sidebar-folder__header--nav-active`

## Testes

`cargo test --workspace`: 82 (inalterado). `cd ui && cargo test --lib`:
75 (+4). Total 157.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: fluxo completo testado (ativar,
colapsar/expandir/entrar em pasta, abrir página com Enter, sair com
Escape sem fechar a página, clique de mouse ainda funciona depois).

## Notas

Bug encontrado e corrigido na própria validação: Escape sem
`stop_propagation` borbulhava e também deselecionava a página aberta
(caso especial de Escape do `app.rs`, fora do `GlobalKeymap`). Detalhes
no arquivo de task.

Próximo: navegação de abas via teclado (107).
