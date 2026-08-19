---
id: "161"
titulo: "Escape em modal desseleciona a página aberta"
status: pending
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

- [ ] `Modal` chama `e.stop_propagation()` no Escape (e no Tab, pelo
      mesmo motivo — o trap de foco não deveria disparar atalho global)
- [ ] Com um modal aberto, Escape fecha SÓ o modal: a página continua
      aberta e o editor continua no lugar
- [ ] Sem modal aberto, Escape continua desselecionando a página
      (comportamento existente, não é o alvo deste ciclo)
- [ ] Auditar os outros modais/menus com Escape próprio pela mesma
      falta de `stop_propagation`: `DialogHost`, painel de propriedades,
      paleta de comandos, menu de slash, popup de wikilink, menus
      dropdown (`menu_keyboard.rs`)
- [ ] Validação ao vivo (MCP `tauri`): abrir o modal de configuração de
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

Achado durante a validação do ciclo 154, mas é anterior a ele — vale
pra todo modal do app desde o ciclo 124. Virou task própria em vez de
remendo dentro do 154 pela regra de isolamento do `cycles/README.md`.
