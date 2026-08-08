---
id: "104"
titulo: "Generaliza padrao de keymap e modal de captura"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Generaliza o padrão de keymap + modal de captura

## Objetivo

Primeiro ciclo do tema "navegação 100% via teclado" (ver
`/home/elis/.claude/plans/agent-os-e-teclado.md`). O `VimKeymap` (ciclo
092) já tem exatamente o padrão certo (struct de `ação -> tecla`,
`labeled_fields`/`set_by_label`, modal que captura a próxima tecla
pressionada) — mas está hardcoded pro vim mode, sem nada reaproveitável
pra um keymap DIFERENTE (o `GlobalKeymap` do ciclo 105). Este ciclo
extrai só a PARTE REUTILIZÁVEL (o modal de captura), SEM mudar
comportamento nenhum do vim mode existente.

## Critérios de aceite

- [ ] Novo componente `ui/src/components/keymap_capture_modal.rs` (ou
      nome similar): recebe `title: String`, `fields: Vec<(&'static
      str, String)>` (rótulo + tecla atual) e `on_change: Callback<
      (String, String)>` (rótulo, tecla nova) — genérico, não sabe nada
      de vim mode nem de `VimKeymap` especificamente
- [ ] `VimSettingsModal` passa a usar esse componente genérico por
      baixo, mantendo EXATAMENTE o comportamento/aparência atual (teste
      manual: reatribuir uma tecla do vim mode continua funcionando
      igual)
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
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

- Criar o `GlobalKeymap` em si — é o ciclo 105, este é só a extração/
  generalização do componente de UI
- Mudar a aparência/UX do modal de configuração do vim mode — deve
  ficar visualmente idêntico depois da extração

## Notas

Ciclo pequeno e de baixo risco de propósito — é puramente um refactor
(extrair um componente genérico de um específico), sem funcionalidade
nova visível ainda. Serve de base pro ciclo 105 não precisar duplicar a
lógica de "capturar a próxima tecla pressionada" numa segunda cópia do
modal.
