---
title: "Ciclo 133 — Nav-mode: motor genérico e regiões de topo"
type: ciclo
ciclo: "133"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 133 — Nav-mode: motor genérico e regiões de topo

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Nav-mode: motor genérico e regiões de topo

## Objetivo

Primeiro ciclo do modo de navegação hierárquico por teclado pedido
pelo usuário (planejado em `/home/elis/.claude/plans/jaunty-tinkering-beaver.md`):
motor genérico baseado em atributos `data-nav-*` do DOM + navegação
entre as 4 regiões de topo (header, sidebar, tab-bar, editor), com
Enter pra descer/delegar, Backspace pra subir um nível, Escape pra
voltar aos wrappers principais (ou sair de vez se já estava lá).

## Critérios de aceite

- [x] `ui/src/nav_mode.rs` (novo) — `items_in_group`, `index_of`,
      `focus_item` (com scroll-into-view `Nearest`), `delegate_of`,
      `group_of`, todos consultando o DOM ao vivo via `data-nav-item`/
      `data-nav-parent`/`data-nav-group`/`data-nav-delegate`
- [x] `GlobalKeymap` ganha `toggle_nav_mode` (17→18 campos, testes
      atualizados) — capacidade persistida em localStorage
      (`nav_mode_enabled`/`save_nav_mode_enabled`/
      `load_nav_mode_enabled`, mesmo padrão do vim mode)
- [x] `app.rs`: `nav_mode_active`/`nav_stack` (sessão sempre
      transitória, não persiste), extensão do `onkeydown` já existente
      — primeira seta (capacidade ligada, fora de alvo editável) inicia
      a sessão; setas navegam irmãos com wrap-around; Enter desce num
      grupo, delega (sidebar/editor) ou ativa uma folha via `.click()`;
      Backspace sobe um nível; Escape limpa a pilha OU sai da sessão se
      já estava na raiz
- [x] `data-nav-*` em: `header.header-bar` (grupo, 4 botões como
      itens), `aside.app-sidebar` (delegate pra sidebar), `.tab-bar`
      (grupo, cada aba é item — ganhou de brinde `tabindex`+Enter/
      Espaço que NUNCA teve), `.app-main-panel` (delegate pro editor,
      mesma query do `focus_editor`/Ctrl+L de sempre)
- [x] Badge de indicador (`.nav-mode-badge`, canto inferior direito,
      só durante sessão ativa) + destaque do container do grupo atual
      (`.nav-mode__region-active`, imperativo via `use_effect_with`)
- [x] `cd ui && cargo test --lib` (81), `cargo test --workspace` (116),
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: ciclo completo — Ctrl+R liga a
      capacidade, seta inicia a sessão, 4 regiões de topo com
      wrap-around nos dois sentidos, Enter desce no header (badge +
      destaque corretos), setas dentro do header, Backspace sobe um
      nível, Enter+Escape+Escape (desce de novo, Escape volta pra raiz
      EM UM PRESS, Escape de novo sai da sessão), delegate pra sidebar
      (foco cai exatamente onde Ctrl+E cairia, sidebar assume e sua
      própria seta funciona), delegate pro editor (foco no
      `.editor__wysiwyg`), grupo tab-bar (desce, Enter na aba ativa
      ela — sessão continua ativa depois de ativar uma folha, só
      delegates saem da sessão), capacidade desligada = setas voltam a
      não fazer nada (sem regressão pra quem não liga o recurso),
      Ctrl+E (sidebar) confirmado continuando a funcionar igual

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Grupos dos componentes de página inteira (kanban/calendário/tabela/
  grafo) e a entrada na cheatsheet — ciclo 134
- Teclado nos embeds inline — ciclo 135
- Supressão do nav-mode com overlays abertos (modal/paleta/etc) e
  validação de coexistência com o vim mode — ciclo 136 (limitação
  conhecida documentada nas Notas abaixo)

## Notas

### Bug real encontrado e corrigido durante a validação

Primeira tentativa: `data-nav-item` nos containers de região
(`<header>`, `<aside>`, `.app-main-panel`, `.tab-bar`) sem `tabindex`
— `.focus()` chamado via `web_sys` FALHA SILENCIOSAMENTE em elementos
não focáveis por padrão (só `<button>`/`<input>`/`<a href>`/elementos
com `tabindex` aceitam foco programático). A primeira seta ligava a
sessão (badge aparecia) mas o foco ficava preso em `.app-root`, sem
nenhum item destacado. Corrigido adicionando `tabindex="0"` nos 4
containers de região. Confirmado via `document.activeElement` antes/
depois do fix.

### Limitação conhecida, adiada pro ciclo 136

Ativar uma folha simples (ex: o botão "⚙" dentro do header, via
`.click()`) NÃO sai da sessão do nav-mode (só delegates saem) —
significa que abrir o menu ⚙ enquanto navegando deixa TANTO o menu
quanto o nav-mode "ouvindo" Escape ao mesmo tempo; um Escape ali
fecharia o menu E subiria um nível do nav-mode na mesma tecla. Não
testado ao vivo neste ciclo (nenhum item do header além dos botões
simples foi ativado via Enter durante a validação); tratamento
explícito (suprimir nav-mode enquanto QUALQUER overlay estiver aberto)
é o próprio escopo do ciclo 136, não uma surpresa nova.

### Decisão de design confirmada na prática

`tabbar` e `editor` são dois itens IRMÃOS na raiz (ambos
`data-nav-parent="root"`) mesmo `.tab-bar` vivendo, no DOM real, como
FILHO de `.app-main-panel` (que carrega `data-nav-item="editor"`) —
intencional: a hierarquia de navegação é definida pelos atributos
`data-nav-*`, não pela profundidade real do DOM, exatamente como
previsto na decisão de arquitetura nº 1 do plano. Validado
funcionando sem problema (tabbar aparece como 4º item da raiz,
alcançável direto sem precisar "entrar" no editor primeiro).

## Resultado

# Ciclo 133 - done

## Resumo

Primeiro ciclo do modo de navegação hierárquico por teclado: motor
genérico (`ui/src/nav_mode.rs`) baseado em atributos `data-nav-*` do
DOM, consultados ao vivo — sem árvore Rust hardcoded, pra qualquer
componente futuro poder participar só marcando o próprio markup.
Cobre as 4 regiões de topo (header, sidebar, tab-bar, editor) com
delegação pros sistemas de navegação já existentes (sidebar) em vez
de reescrevê-los.

## Arquivos criados/modificados

- `ui/src/nav_mode.rs` (novo)
- `ui/src/lib.rs` — registra o módulo
- `ui/src/state.rs` — `GlobalKeymap.toggle_nav_mode` + persistência
  `nav_mode_enabled`
- `ui/src/app.rs` — estado da sessão, extensão do `onkeydown`, badge,
  destaque de região
- `ui/src/components/header_bar.rs` — `data-nav-*` no header e 4
  botões
- `ui/src/components/sidebar.rs` — `data-nav-*` delegate
- `ui/src/components/tab_bar.rs` — `data-nav-*` + tabindex/Enter/
  Espaço (gap de teclado que a tab-bar nunca teve)
- `ui/src/styles/main.css` — badge + destaque de região

## Testes

`cd ui && cargo test --lib`: 81. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: ciclo completo de navegação
testado — 4 regiões com wrap-around, descer/subir níveis, os dois
comandos de Escape (raiz vs. sair), delegate pra sidebar E pro editor
confirmados ponta-a-ponta, ativação de folha (aba) sem sair da sessão,
capacidade desligada sem afetar comportamento default, Ctrl+E
confirmado sem regressão.

## Notas

Bug real encontrado e corrigido: faltava `tabindex="0"` nos 4
containers de região — `.focus()` via `web_sys` falha silenciosamente
em elementos não focáveis por padrão. Ver Notas detalhadas no arquivo
de task pra esse achado e pra uma limitação conhecida (interação com
overlays abertos) explicitamente adiada pro ciclo 136.

Próximo: ciclo 134 — grupos dos componentes de página inteira
(kanban/calendário/tabela/grafo) + entrada na cheatsheet.
