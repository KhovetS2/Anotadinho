---
id: "107"
titulo: "Navegacao de abas via teclado"
status: pending
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: ["105"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Navegação de abas via teclado

## Objetivo

Quarto ciclo do tema "navegação 100% via teclado". Hoje só existe
`Ctrl+W` (próxima aba, cíclico). Este ciclo adiciona "aba anterior"
(já é uma ação do `GlobalKeymap` do ciclo 105, faltando implementação)
e "pular direto pra aba N" (`Ctrl+1`..`Ctrl+9`, fixo — não faz sentido
customizar 9 binds separados no `GlobalKeymap`).

## Critérios de aceite

- [ ] "Próxima aba"/"Aba anterior" (`GlobalKeymap`) navegam
      ciclicamente pra frente/trás em `open_tabs` (próxima já existe via
      `Ctrl+W`; anterior é nova)
- [ ] `Ctrl+1` a `Ctrl+9` pulam direto pra aba de índice 0-8 (se
      existir aquele índice; sem efeito se não tiver aba suficiente) —
      fixo, não faz parte do `GlobalKeymap` customizável (mesmo padrão
      de navegador/editor de código)
- [ ] `ui/src/components/tab_bar.rs` ganha um indicador visual sutil do
      número de cada aba (ex: tooltip ou badge pequeno) pra descobrir
      qual `Ctrl+N` pula pra qual
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

- Reordenar abas por teclado (drag continua sendo só mouse, se existir
  — confirmar se já existe reorder de abas antes de decidir se entra
  aqui)
- `Ctrl+1..9` customizável — fixo de propósito (convenção já
  estabelecida em outras ferramentas, menos uma coisa pra configurar)

## Notas

Depende do ciclo 105 pro dispatcher de `GlobalKeymap` já existir (esse
ciclo só adiciona as ações que faltam + a lógica fixa de `Ctrl+1..9`,
que fica FORA do keymap customizável, direto no dispatcher do
`app.rs`).
