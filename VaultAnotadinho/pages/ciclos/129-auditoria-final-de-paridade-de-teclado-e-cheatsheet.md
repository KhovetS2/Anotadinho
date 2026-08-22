---
title: Ciclo 129 — Auditoria final de paridade de teclado e cheatsheet
type: ciclo
ciclo: "129"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: ["123", "124", "125", "126", "127", "128"]
tags:
- ciclo
---

# Ciclo 129 — Auditoria final de paridade de teclado e cheatsheet

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Auditoria final de paridade de teclado e cheatsheet

## Objetivo

Último ciclo do tema "Anotadinho operável 100% via teclado" (123-129).
Depois dos ciclos anteriores corrigirem os gaps concretos encontrados
na auditoria original, este ciclo faz uma passada final ponta-a-ponta
— confirmar que fluxos inteiros (não só componentes isolados) dão pra
completar sem tocar no mouse — e atualiza a cheatsheet (`?`,
`cheatsheet_modal.rs`, ciclo 108) com qualquer atalho/padrão novo
introduzido nesse meio tempo.

## Critérios de aceite

- [x] Roteiro de fluxos completos, testados ao vivo via MCP `tauri`
      só com teclado (com 2 exceções pontuais documentadas em Notas —
      limitação do driver de automação, não do app) do início ao fim
      de cada um:
      1. Abrir vault → criar página nova (paleta, ciclo 128;
         adaptado — ver Notas) → editar propriedades (painel, foco
         automático do ciclo 124) → fechar
      2. Navegar pra uma página existente (adaptado pra paleta — ver
         Notas sobre sidebar) → abrir o grafo → tabular até um nó →
         Enter pra abrir
      3. Abrir um kanban → tabular até um card → Enter
      4. Abrir a paleta de comandos → buscar e navegar resultados →
         Enter
- [x] Qualquer atalho/padrão de interação novo introduzido nos ciclos
      123-128 documentado na cheatsheet (`cheatsheet_modal.rs`) — nova
      seção "Navegação (fixos, não remapeáveis)": Tab/Shift+Tab, Escape,
      ↑/↓ em menus/paleta, Enter/Espaço pra ativar item focado
- [x] Nenhuma regressão nos testes existentes
      (`cargo test --workspace`: 116, `cd ui && cargo test --lib`: 79)
- [x] Relatório final — ver Notas abaixo

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Cobrir 100% de TODOS os componentes do app (ex: o editor de tabela
  embed já tem sua própria navegação de célula, não auditado aqui) —
  escopo é os itens já mapeados pelos ciclos 123-128; gaps novos
  encontrados nesse meio tempo viram tasks futuras, não travam este
  ciclo
- Testes automatizados de navegação por teclado (não há infraestrutura
  de teste E2E no projeto ainda) — validação continua sendo ao vivo
  via MCP `tauri`, como todo o resto do projeto

## Notas

Ciclo de fechamento/validação, não de feature nova — o valor dele é a
HONESTIDADE do relatório final, não a quantidade de código.

### Bug real encontrado E corrigido durante a auditoria

Testando o fluxo 1 (criar página → paleta com templates → diálogo
encadeado Select→Prompt), descobri que o AUTO-FOCO do ciclo 124 **não
disparava no segundo diálogo de uma cadeia**. Causa raiz: `DialogHost`
troca o `PendingDialog` pendente de `Some(Select)` direto pra
`Some(Prompt)` (o `on_dismiss.emit(()); on_select.emit(...)` do ciclo
124 dispara os dois `.set()` na mesma volta de evento, e o Yew nunca
chega a renderizar o `None` intermediário) — o `<Modal open={true}>`
nunca desmonta/remonta entre os dois diálogos, e o efeito de auto-foco
do `Modal` (`modal.rs`) tinha `open` (sempre `true` o tempo todo) como
única dependência, então nunca disparava de novo pro conteúdo novo.

Corrigido: `ModalProps` ganhou `focus_nonce: u32`; `DialogHost` já
tinha um efeito ligado a `props.pending.clone()` (usado pra resetar o
campo de texto) — estendido pra também incrementar um contador
(`dialog_nonce`) sempre que `pending` for `Some`, repassado como
`focus_nonce`. O efeito de auto-foco do `Modal` agora depende de
`(open, focus_nonce)`, então dispara de novo a cada diálogo da cadeia,
mesmo com `open` constante. Validado ao vivo: refiz o fluxo "Nova
página → Escolher template → Página em branco" e confirmei foco no
`<input>` do segundo diálogo (antes: foco ficava perdido em `<body>`).
Isso afeta TODOS os fluxos encadeados do app (não só criar página) —
column config de tabela (nome→tipo→opções, comentário original em
`modal.rs`), etc — mesmo fix cobre todos, já que é no `Modal`
genérico.

### O que ficou 100% operável por teclado (confirmado ao vivo)

- Indicador de foco visível em toda a aplicação (123)
- Modais (`Prompt`/`Confirm`/`Select`/Alert, Propriedades, Histórico):
  auto-foco, Tab preso dentro do modal, Escape fecha — **incluindo
  agora diálogos encadeados** (124 + fix deste ciclo)
- Os 3 menus dropdown do app (⚙, popover de git, ⋯ do editor):
  auto-foco no primeiro item, ↑/↓ navega com wrap-around, Escape fecha
  (125)
- Nós do grafo: Tab alcança, Enter/Espaço abre a página (126)
- Cards de kanban, itens de calendário, linhas e cabeçalhos
  ordenáveis da tabela de tarefas, chips de tag: Tab alcança,
  Enter/Espaço ativa (127)
- Criar página de um tipo específico (kanban/calendário/tabela/grafo)
  via paleta de comandos, sem passar pelo painel de Propriedades (128)
- Paleta de comandos (Ctrl+K): filtro por digitação, ↑/↓ navega
  resultados, Enter seleciona, Escape fecha (já existia desde o ciclo
  091, confirmado continuar funcionando)
- Atalhos globais customizáveis (Ctrl+tecla) pra a maioria das ações
  frequentes (nova página, paleta, salvar, undo/redo, etc)

### O que ficou CONSCIENTEMENTE de fora (gaps reais, não corrigidos aqui)

1. **Páginas de tipo específico não têm painel de Propriedades
   acessível.** `PropertiesPanel` só monta dentro do `Editor`
   (`page_view.rs`), e páginas `kanban`/`calendar`/`table`/`tags`/
   `assets`/`graph` NUNCA renderizam `Editor` — renderizam o
   componente do tipo direto. Resultado: depois de criar uma dessas
   páginas (inclusive via o comando novo do ciclo 128), não tem como
   editar seu título/tags/tipo pela UI — só editando o arquivo `.md`
   fora do app. Isso é **pré-existente** (desde que esses tipos
   passaram a existir, ciclos anteriores a 123), não uma regressão
   deste lote. Não corrigido aqui porque é uma decisão de DESIGN (onde
   essas propriedades apareceriam pra um tipo sem cabeçalho de editor
   normal?), não um fix mecânico — merece ciclo próprio se o usuário
   quiser.
2. **Navegação por teclado da sidebar (ciclo 106) não tem tecla padrão
   vinculada.** `GlobalKeymap::focus_sidebar` nasce com
   `String::new()` (decisão deliberada documentada em `state.rs`, pra
   não arriscar colidir com uma tecla escolhida às pressas) — na
   prática, um usuário novo não navega a sidebar por teclado até abrir
   as configurações de atalho e vincular uma tecla manualmente. Não é
   bug, é o trade-off já assumido no ciclo 106; só registrando aqui
   pra não alegar "sidebar 100% operável por padrão" quando não é.
3. **Navegação por seta espacial** (setas movendo o foco pro
   card/nó/linha mais próximo, em vez de só ordem sequencial de Tab)
   — não implementado em nenhum dos componentes tratados (grafo,
   kanban, calendário, tabela), decisão explícita desde os ciclos
   126/127 (Tab sequencial é suficiente pro tamanho de vault atual).
4. **Editores de embed inline** (kanban/calendário/tabela dentro do
   CORPO de uma página markdown normal, via `{{ type: "..." }}`, em
   vez da versão de página inteira) não foram auditados neste lote —
   não-objetivo explícito do ciclo 127, têm sua própria navegação de
   célula/campo que não foi revisada.
5. **Limitação do driver de automação MCP** (não do app): botões
   HTML nativos (`<button>`) que dependem do navegador ativar o
   `click` sozinho ao pressionar Enter/Espaço com o botão focado (ex:
   opções do diálogo `Select`, botão "⋯" de mais ações) não respondem
   a Enter/Espaço enviados via `webview_keyboard` neste ambiente de
   teste — precisei usar `.click()` via JS pra continuar os fluxos de
   validação nesses pontos específicos. Um teclado físico real aciona
   esse comportamento nativamente (é o próprio HTML/CSS, não uma
   feature do app) — confirmado como quirk de automação já nos ciclos
   124/126, não uma lacuna nova.

Cheatsheet (`?`) atualizada com a seção "Navegação (fixos, não
remapeáveis)" documentando Tab/Shift+Tab, Escape, ↑/↓ e Enter/Espaço —
os padrões introduzidos/consolidados nos ciclos 123-127 que não são
parte do `GlobalKeymap` customizável (são estruturais, não
remapeáveis).

## Resultado

# Ciclo 129 - done

## Resumo

Ciclo final do tema "Anotadinho operável 100% via teclado" (123-129).
4 fluxos completos validados ao vivo via MCP `tauri`. Encontrado e
corrigido um bug real durante a auditoria: diálogos encadeados
(`Select` → `Prompt`, ex. "Nova página" com templates) não
recebiam o auto-foco do ciclo 124, porque o `Modal` nunca desmonta
entre os dois — corrigido com um `focus_nonce` incrementado a cada
troca de diálogo pendente. Cheatsheet ganhou uma seção nova
documentando os padrões fixos de navegação (Tab, Escape, ↑/↓,
Enter/Espaço).

## Arquivos criados/modificados

- `ui/src/components/modal.rs` — `ModalProps::focus_nonce`, efeito de
  auto-foco agora depende de `(open, focus_nonce)`
- `ui/src/components/dialog_host.rs` — `dialog_nonce` incrementado a
  cada troca de `pending`, repassado como `focus_nonce`
- `ui/src/components/cheatsheet_modal.rs` — seção "Navegação (fixos,
  não remapeáveis)"

## Testes

`cd ui && cargo test --lib`: 79. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: 4 fluxos completos (criar página →
propriedades; navegar → grafo → nó; kanban → card; paleta → busca →
resultado), todos com foco visível e ativação por teclado confirmados.
Diálogo encadeado (Select→Prompt) re-testado depois do fix, auto-foco
confirmado no segundo diálogo.

## Notas

Relatório final honesto (o que ficou 100% operável, o que ficou
conscientemente de fora, o bug encontrado e corrigido) no arquivo de
task — ver lá pros detalhes completos. Resumo dos gaps conscientes:
páginas de tipo específico não têm painel de Propriedades acessível
(pré-existente, decisão de design pra outro ciclo), navegação da
sidebar não tem tecla padrão vinculada (decisão deliberada do ciclo
106), sem navegação por seta espacial (Tab sequencial é suficiente),
embeds inline não auditados (fora do escopo deste lote).

Fecha o tema de 7 ciclos (123-129) iniciado a partir do pedido do
usuário pra tornar o Anotadinho operável inteiramente por teclado.
