---
id: "135"
titulo: "Nav-mode: teclado nos embeds inline"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Nav-mode: teclado nos embeds inline

## Objetivo

Terceiro ciclo do tema pedido pelo usuário: embeds inline (kanban/
calendário/tabela dentro do CORPO de uma página, via `{{ type: "X" }}`)
nunca tiveram NENHUM suporte de teclado — só mousedown/mouseup/
mouseenter pra drag-and-drop. Retrofit da ativação por Enter/Espaço
reaproveitando `keyboard_activate.rs`, na mesma ação que o clique (sem
arrastar) já dispara hoje.

## Critérios de aceite

- [x] `inline_kanban.rs`: card ganha `tabindex="0"` + Enter/Espaço
      abrindo o `CardDetailModal` (mesma ação de soltar o mouse no
      próprio card sem arrastar)
- [x] `inline_calendar.rs`: 3 pontos — barra somente-leitura do modo
      Vault (navega pra página de origem), item da agenda somente-
      leitura do modo Vault (mesma navegação), barra editável da
      visão Mês (abre `EventDetailModal`) — todos com `tabindex="0"` +
      Enter/Espaço
- [x] `inline_table.rs`: os 4 tipos de célula que usam um `<span
      onclick>` não focável pra abrir um menu (Date/PageLink/
      MultiSelect/Select) ganham `tabindex="0"` + Enter/Espaço
      abrindo o mesmo menu — checkbox/number/url/text já eram
      nativamente focáveis (input/button/textarea), sem mudança
      necessária
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri` na página `exemplos-embeds`:
      card do kanban inline (Enter abriu "Tarefa 1"), barra do
      calendário inline (Enter abriu "Evento"), date-chip da tabela
      inline (Enter abriu o date picker), badge de Select da tabela
      inline (Enter abriu o menu de opções) — todos confirmados

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Reordenar card/coluna/evento por teclado (drag-and-drop) — continua
  só mouse, mesma limitação dos componentes de página inteira
- Visões Semana/Dia do calendário inline (blocos com horário e alças
  de redimensionar) — significativamente mais complexas
  estruturalmente (múltiplas funções de render, resize por mouse) que
  a visão Mês (padrão); adiado, ver Notas
- Navegar por seta DENTRO de um menu/dropdown já aberto de uma célula
  da tabela (`.table-select-menu__item`) — o gatilho pra ABRIR o menu
  agora é alcançável por teclado, mas os itens do menu em si ainda só
  respondem a clique; precisaria do mesmo padrão de auto-foco+seta já
  usado em `menu_keyboard.rs` (ciclo 125), fora do escopo deste ciclo
- `data-nav-item`/`data-nav-parent` do nav-mode nos embeds — ver Notas
  (decisão consciente de NÃO adicionar agora)

## Notas

### Ajuste de escopo consciente: sem atributos `data-nav-*` nos embeds

O plano original previa marcar os embeds com `data-nav-item`/
`data-nav-parent` pro nav-mode. Na prática, descobri durante a
implementação que isso não teria efeito nenhum: o delegate `"editor"`
(ciclos 133/134) sempre foca `.editor__wysiwyg` PRIMEIRO quando esse
elemento existe — e ele SEMPRE existe numa página de texto normal com
embeds (o embed vive DENTRO do corpo editável). Marcar os cards do
embed como filhos de um grupo "editor" não teria como ser alcançado
pelo motor do nav-mode, que nunca trata "editor" como um grupo
navegável (só como delegate/folha).

Isso não é uma lacuna real, porque uma vez que o nav-mode foca o
`.editor__wysiwyg` (ou o usuário chega lá via clique/Ctrl+L), o **Tab
nativo do navegador** já alcança os cards/barras/células do embed em
ordem de documento — é exatamente o que este ciclo entrega (`tabindex`
nos elementos certos). Adicionar os atributos `data-nav-*` sem um
mecanismo de "grupo dentro do editor" pra consumi-los seria marcação
morta. Deixado de fora deliberadamente; se o usuário sentir falta de
pular DIRETO pra dentro de um embed específico (em vez de Tab
sequencial a partir do início do texto), isso vira uma decisão de
design nova pro ciclo 136 ou além — não uma correção de bug.

### Escopo do calendário: só visão Mês

`inline_calendar.rs` tem MÚLTIPLAS funções de render pras visões Mês
(padrão)/Semana/Dia, cada uma com sua própria lógica de barra —
Semana/Dia usam blocos posicionados por horário com alças de
redimensionar (`calendar-grid__resize-handle`), estrutura bem mais
complexa. Cobri as barras da visão Mês (padrão) + a agenda somente-
leitura do modo Vault (usada tanto em Mês quanto Semana/Dia nesse
modo). Blocos com horário da visão Semana/Dia (modo Manual) ficam de
fora — não reportados como bug pelo usuário, e a complexidade
estrutural não compensava o risco nesse ciclo.
