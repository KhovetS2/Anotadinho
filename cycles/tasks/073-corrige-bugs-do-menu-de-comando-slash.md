---
id: "073"
titulo: "Corrige bugs do menu de comando slash"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Corrige bugs do menu de comando slash

## Objetivo

Usuário reportou dois bugs no menu `/` do editor: (1) navegar com o
teclado pra um item fora da área visível da lista não rola o menu pra
mostrá-lo; (2) selecionar uma opção com clique do mouse não aplica ela no
lugar certo.

## Critérios de aceite

- [x] Navegar com `ArrowUp`/`ArrowDown` rola a lista (`scrollIntoView`
      com `block: nearest`) pra manter o item ativo sempre visível
- [x] Clicar um item com o mouse aplica AQUELE item, não o que estava
      destacado por último via teclado — bug raiz real encontrado:
      `select_slash` lia `*slash_idx` (estado de navegação por teclado)
      em vez de receber a posição do item clicado
- [x] `onmousedown` do item do menu chama `prevent_default()` — sem isso
      o navegador rouba o foco/seleção de dentro do contenteditable antes
      do `onclick` disparar, e o `execCommand("insertHTML")` não tinha
      onde inserir

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mudar o menu pra inserir literalmente `/texto` no documento (a
  arquitetura atual mantém isso só como estado interno, mostrado no
  cabeçalho flutuante do menu, nunca tocando o HTML real) — é uma
  mudança de design maior, não um bug; fica registrada separada no
  tracker (#64 ainda cobre essa parte, mas não foi escopo deste ciclo)

## Notas

Dois bugs distintos, achados durante a investigação:

1. **Scroll-into-view**: não existia nenhum mecanismo de rolar a lista
   ao navegar — corrigido com um `NodeRef` no item ativo +
   `scroll_into_view_with_scroll_into_view_options` (`block: nearest`)
   num efeito disparado por `(*slash_idx, *slash_open)`.

2. **Clique aplica o item errado**: raiz real (achada lendo o código, não
   só suposição) — `select_slash` sempre lia `*slash_idx` (o índice
   navegado por teclado) pra decidir qual item aplicar, IGNORANDO
   completamente qual `<div>` foi clicado. Bastava navegar com seta até
   um item e depois clicar em OUTRO com o mouse que o item errado (o
   destacado por teclado) era inserido. Corrigido mudando `select_slash`
   de `Callback<()>` pra `Callback<usize>`, recebendo a posição do item
   na lista filtrada — `onclick` de cada item passa a própria posição
   (`vi`), `Enter` continua passando `*slash_idx`.

   Também corrigido de passagem o problema de foco/seleção: `mousedown`
   num item (antes do `onclick`) colapsa a seleção do navegador pra fora
   do contenteditable se não tiver `prevent_default()` — isso fazia o
   `execCommand("insertHTML")` não ter onde inserir. Confirmado ao vivo
   via MCP que a seleção permanece dentro do editor depois do
   `mousedown` num item do menu.

Validado ao vivo via MCP `tauri`: abrir o menu, navegar com seta até
"Título 3" (destacado), clicar com o mouse em "Linha" (item diferente,
mais abaixo) — o HTML resultante teve `<hr>` inserido no lugar certo
(dentro do `<li>` onde estava o cursor), confirmando que o item CLICADO
foi aplicado, não o navegado. Scroll-into-view confirmado navegando até o
item "Diagrama" (fora da área visível inicial) e vendo a lista rolar pra
mostrá-lo.
