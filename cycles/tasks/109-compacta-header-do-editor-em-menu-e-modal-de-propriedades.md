---
id: "109"
titulo: "Compacta header do editor em menu e modal de propriedades"
status: done
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: ["099"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Compacta header do editor em menu e modal de propriedades

## Objetivo

Pedido direto do usuário: o header do editor acumulou muitos botões
soltos (🏠, Excluir, ⬇ Exportar HTML, Salvar) e o painel de
propriedades (ciclo 099) ocupa uma faixa fixa acima do corpo mesmo
quando não está em uso. Este ciclo (a) move Excluir/Exportar
HTML/Definir como início/Propriedades pra dentro de um menu "⋯"
compacto, mantendo só "Salvar" como botão principal sempre visível;
(b) troca o painel de propriedades inline por um MODAL, aberto a
partir desse menu; (c) na aba fixa da página inicial (🏠, ciclo 089),
mostra só o ícone de casa fixo em vez do título da página.

## Critérios de aceite

- [x] `ui/src/components/editor.rs`: header do editor com só o título
      + botão "Salvar" + botão de menu "⋯"; o menu (mesmo padrão visual
      de popover do menu ⚙ da `HeaderBar`) lista: "🏠 Definir/remover
      como início", "Propriedades...", "⬇ Exportar HTML", "Excluir"
      (item de destaque/perigo, mantém a confirmação existente)
- [x] `ui/src/components/properties_panel.rs` (ou um wrapper novo)
      passa a renderizar DENTRO de um `Modal` em vez de inline
      colapsável — aberto pelo item "Propriedades..." do menu, mesmos
      campos/comportamento de edição de hoje (título/tipo/tags/extra)
- [x] Aba fixa da página inicial na `TabBar`: mostra só "🏠" (sem o
      título da página), com `title="..."` (tooltip HTML nativo) com o
      nome de verdade pra continuar identificável ao passar o mouse
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

- Mudar o CONTEÚDO/campos do painel de propriedades — só a moldura
  (inline → modal), lógica de edição idêntica ao ciclo 099
- Menu "⋯" customizável ou reordenável — ordem fixa dos 4 itens
- Mexer no menu ⚙ da `HeaderBar` (global) — esse ciclo é só do header
  do `Editor` (por página) e da `TabBar`

## Notas

Reaproveita o padrão de popover já usado no menu ⚙ da `HeaderBar`
(`header-menu-wrapper`/`header-menu`/`header-menu__item`, fechar ao
clicar fora/Escape) — mesmas classes CSS, evitando duplicar o padrão
visual.

O ícone de casa fixo na aba precisa saber QUAL aba é a página inicial
— já existe `state::load_home_page(vault_path)` (ciclo 089) comparando
contra `path` de cada tab.

Mudança de arquitetura necessária: o estado "qual página é a inicial"
vivia só DENTRO do `Editor` (`is_home`/`toggle_home` locais, ciclo
089). A `TabBar` é IRMÃ do `Editor` (ambos filhos de `App`), não
descendente — pra ela também saber mostrar o 🏠 na aba certa, o estado
precisou subir pro `App` (`home_page: UseStateHandle<Option<String>>`
+ `on_toggle_home: Callback<String>`), repassado como prop por
`PageView` até o `Editor` (que agora só DERIVA `is_home` do prop, sem
estado próprio). Efeito colateral positivo: agora qualquer componente
futuro que precise saber "qual é a página inicial" só precisa de um
prop, não duplica a leitura de `localStorage`.

`PropertiesPanel` perdeu o `<details>/<summary>` (não faz mais sentido
colapsável dentro de um modal que já tem cabeçalho próprio) — vira só
o conteúdo (`.properties-panel__body`), a moldura é o `Modal`
genérico.

Validado ao vivo via MCP `tauri`: aba da página inicial mostra 🏠 (não
mais "incio"); aba de página normal continua com o título; menu "⋯"
lista os 4 itens corretos com o rótulo de início dinâmico ("Remover"
quando já é a inicial); "Propriedades..." abre o modal com os mesmos
campos de antes; "Excluir" ainda passa pela confirmação (não apaga
direto).
