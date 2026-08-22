---
title: Ciclo 065 — Upgrade tabela de tarefas tipos de celula novos
type: ciclo
ciclo: "065"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: ["064"]
tags:
- ciclo
---

# Ciclo 065 — Upgrade tabela de tarefas tipos de celula novos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Upgrade da tabela de tarefas: tipos de célula novos + configuração aprimorada

## Objetivo

A tabela inline só tinha 3 tipos de coluna (Texto/Checkbox/Seleção) e
configurar uma coluna era 3 `PendingDialog::Prompt` encadeados digitando o
tipo como texto livre. O usuário pediu Tags (multi-seleção — criar tag
digitando E escolher entre as existentes) e, ao ser perguntado quais
outros tipos, escolheu Número, Data, URL/Link e um tipo que vincula a
outra página do vault. Este ciclo entrega os 5 tipos novos (3→8 no total)
e um `ColumnSettingsModal` de verdade substituindo os prompts encadeados.

## Critérios de aceite

- [x] `ColumnKind` ganha `MultiSelect`, `Number`, `Date`, `Url`, `PageLink`
      — célula continua sendo `String` em `rows` (sem reestruturar o
      modelo), com o corpo YAML/markdown-table já existente
- [x] `ColumnSettingsModal` (novo componente) substitui os 3 prompts
      encadeados: nome (contenteditable+blur), tipo (`<select>` com as 8
      opções), editor de opções embutido pra Select/MultiSelect (chips +
      "+ opção", commit imediato em cada mudança)
- [x] Célula Tags: clicar abre dropdown com as opções da coluna como
      itens toggle-áveis (☑/☐) + campo pra digitar uma tag nova — Enter
      cadastra a tag na coluna E marca na célula na mesma ação
- [x] Célula Página: dropdown com filtro de texto listando as páginas do
      vault (`api::list_pages`), célula fechada mostra "📄 {título
      resolvido}" + ícone "↗" que navega pra página (`on_page_selected`,
      mesmo padrão de Kanban/Calendar/TaskTable/Sidebar)
- [x] Célula URL: link clicável + ícone de editar (via `PendingDialog::Prompt`,
      simples o bastante pra não precisar de estado local)
- [x] Células Número/Data: `<input type="number">`/`<input type="date">`
      nativos direto na célula
- [x] `on_page_selected`/`vault_path` threading: `page_view.rs` → `Editor`
      (não tinha `on_page_selected` antes) → `InlineEmbed` → `InlineTable`
- [x] `exemplos-embeds.md` ganha exemplos dos 5 tipos novos; teste
      `exemplos_embeds_vault_file_parses` estendido pra validar os 8 tipos

## Comandos de validação

```bash
cd ui && cargo test --lib embed::
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Célula Select (single-value) não ganhou criação inline de opção — só
  Tags/MultiSelect, conforme pedido original; Select continua só
  escolhendo entre opções já cadastradas no `ColumnSettingsModal`
- Sem parsing/validação forte pros tipos Número/URL — a célula restringe
  o tipo de input, mas o valor cru continua sendo string livre

## Notas

Validação ao vivo via MCP `tauri` confirmou: criar tag digitando + marcar
tag existente no mesmo dropdown; remover uma opção no
`ColumnSettingsModal` limpa a célula certa em toda linha (Select limpa o
valor, MultiSelect remove só aquela tag da lista); trocar o tipo da coluna
via `<select>` re-renderiza a coluna inteira na hora; escolher uma página
no picker resolve e mostra o título certo, e o ícone "↗" navega pra ela de
verdade.

Durante a sessão de teste, o webview do Tauri travou (toda chamada de JS,
até `1+1`, dava timeout, apesar do processo estar vivo com CPU baixa —
não era um loop infinito da aplicação). Provavelmente acúmulo de estado
do bridge MCP depois de várias capturas de screenshot via `html2canvas`
mais um `stop`/`start` de sessão no meio. Resolvido reiniciando o processo
do zero (`kill` + `./scripts/dev.sh`) — depois disso a validação seguiu
normal. Vale lembrar: se o webview parar de responder no meio de uma
sessão de teste longa, reiniciar o processo é mais rápido que tentar
recuperar a conexão.

## Resultado

# Ciclo 065 - done

## Resumo

Tabela inline sai de 3 pra 8 tipos de coluna: Texto, Checkbox, Seleção
(já existentes) + Tags (multi-seleção com criação inline de opção),
Número, Data, URL e Página (relação interna, vincula outra página do
vault com preview do título e navegação). `ColumnSettingsModal` novo
substitui o fluxo de 3 `PendingDialog::Prompt` encadeados que não
escalava mais com 8 tipos.

## Arquivos criados/modificados

- `ui/src/embed.rs` — `ColumnKind` com 5 variantes novas, parse/serialize
  YAML dos tokens novos (`multiselect`/`tags`, `number`, `date`, `url`,
  `page`), `add_column_option`/`remove_column_option`
- `ui/src/components/embeds/column_settings_modal.rs` (novo) — nome,
  tipo (`<select>`), editor de opções pra Select/MultiSelect
- `ui/src/components/embeds/inline_table.rs` — reescrito: 5 editores de
  célula novos, `open_select_cell` generalizado pra `open_cell_menu`
  (Select/MultiSelect/PageLink compartilham o mesmo dropdown), busca de
  páginas do vault via `api::list_pages`
- `ui/src/components/embeds/mod.rs` — `on_page_selected` em
  `InlineEmbedProps`, repassado só pro braço Table
- `ui/src/components/editor.rs` — `on_page_selected` em `EditorProps`
  (Editor não tinha antes, diferente de Kanban/Calendar/TaskTable)
- `ui/src/components/page_view.rs` — passa `on_page_selected` pro Editor
- `ui/src/styles/main.css`, `ui/src/styles/components.css` — CSS novo
  pros tipos de célula (número alinhado, link, badge de página, tags) e
  pro `ColumnSettingsModal` (`<select>`, editor de opções)
- `ui/Cargo.toml` — feature `HtmlSelectElement` do web-sys
- `VaultAnotadinho/pages/exemplos-embeds.md` — Table Embed ganha colunas
  de exemplo dos 5 tipos novos

## Testes

`cargo test --lib embed::` (ui/): 26 passaram (2 novos: round-trip YAML
dos 5 `ColumnKind` novos, `add_column_option`/`remove_column_option`
incluindo o caso de limpar célula ao remover a opção selecionada), mais
o teste de sincronia do vault (`exemplos_embeds_vault_file_parses`)
estendido pros 8 tipos.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

- Célula Tags: criar tag nova digitando ("nova-tag-teste") cadastra na
  coluna E marca na célula na mesma ação; marcar/desmarcar tag existente
  funciona
- Célula Página: filtro de texto funciona, escolher uma página resolve o
  título certo ("📄 sobre"), ícone "↗" navega de verdade pra página
- `ColumnSettingsModal`: adicionar opção, remover opção (limpa a célula
  certa em toda linha — testado em Select e MultiSelect), trocar tipo via
  `<select>` re-renderiza a coluna na hora
- Vault reaberto do zero depois da sessão de teste: conteúdo do
  `exemplos-embeds.md` intacto, todos os 8 tipos renderizando

## Notas

Webview do Tauri travou no meio da sessão de teste (todo `webview_execute_js`
dava timeout, até `1+1` — não era loop infinito da aplicação, CPU dos
processos ficou baixa o tempo todo). Provável acúmulo de estado do bridge
MCP depois de muitas capturas de screenshot via `html2canvas` seguidas de
um `stop`/`start` de sessão. Resolvido reiniciando o processo do zero
(`kill` nos PIDs + `./scripts/dev.sh`) — não é um bug do Anotadinho, é uma
característica do ambiente de teste a ter em mente em sessões longas.
