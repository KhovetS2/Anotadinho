---
id: "161"
titulo: "Escape em modal desseleciona a página aberta"
status: done
criado: 2026-08-19
autor: agente
prioridade: alta
depende_de: []
estima_min: 30
agente_alvo: claude-sonnet
---

# Escape em modal desseleciona a página aberta

## Objetivo

Bug pré-existente, encontrado na validação ao vivo do ciclo 154.
Fechar QUALQUER modal com Escape também desseleciona a página aberta:
o editor some e aparece "Selecione uma página na sidebar", com a aba
ainda lá. Quem estava editando perde o lugar (e, com autosave ligado, o
flush de troca de página grava no meio do caminho).

Causa: `Modal::on_keydown` (`ui/src/components/modal.rs`, ciclo 124)
trata Escape com `e.prevent_default()` mas NÃO `e.stop_propagation()`.
O evento continua subindo até o listener global de
`ui/src/app.rs` (~l.982), que faz `selected_page.set(None)` em qualquer
Escape sem Ctrl.

## Critérios de aceite

- [x] `Modal` chama `e.stop_propagation()` no Escape (e no Tab, pelo
      mesmo motivo — o trap de foco não deveria disparar atalho global)
- [x] Com um modal aberto, Escape fecha SÓ o modal: a página continua
      aberta e o editor continua no lugar
- [x] Sem modal aberto, Escape continua desselecionando a página
      (comportamento existente, não é o alvo deste ciclo)
- [x] Auditoria dos outros donos de Escape. Achado: o problema é MAIOR
      que o modal — todo popup que fecha por listener de `window`
      (seletor de data, de hora, popover de git, menu "⋯" do header e
      do editor, menu de célula da tabela) deixava o Escape seguir pro
      `app.rs`. Menu de slash, popup de wikilink e sidebar já
      paravam a propagação (ciclos 073/082/140); paleta de comandos
      não parava — corrigida
- [x] Validação ao vivo (MCP `tauri`): abrir o modal de configuração de
      consulta, Escape, confirmar que a página segue aberta

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mudar o que Escape faz sem modal aberto
- Refazer o trap de foco do ciclo 124

## Notas

Duas correções, não uma:

1. `Modal` (handler do Yew) ganhou `stop_propagation` no Escape e no
   Tab.
2. Popups que escutam `window` não conseguem parar nada — quando eles
   rodam, o handler do `app.rs` já rodou. Ganharam
   `menu_keyboard::escape_consumer`: listener na fase de CAPTURA da
   `window`, que dispara ANTES do handler delegado do Yew e consome a
   tecla. Efeito colateral bom: o popup mais interno ganha — um seletor
   de data aberto DENTRO de um modal fecha só o seletor, sem levar o
   modal junto (era o risco de simplesmente pôr `stop_propagation` no
   `Modal`, e foi conferido ao vivo).

Validação ao vivo (MCP `tauri`): modal de consulta, menu "⋯" do editor,
paleta de comandos e seletor de data da tabela — todos fecham só a si
mesmos, com a página continuando aberta. Sem nada aberto, Escape ainda
desseleciona a página (comportamento existente, preservado).

Achado durante a validação do ciclo 154, mas é anterior a ele — vale
pra todo modal do app desde o ciclo 124. Virou task própria em vez de
remendo dentro do 154 pela regra de isolamento do `cycles/README.md`.
