---
id: "105"
titulo: "GlobalKeymap atalhos do app inteiro customizaveis"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: ["104"]
estima_min: 150
agente_alvo: claude-sonnet
---

# `GlobalKeymap`: atalhos do app inteiro, customizáveis

## Objetivo

Segundo ciclo do tema "navegação 100% via teclado" — o coração do
pedido. Hoje `app.rs`'s `onkeydown` é uma lista hardcoded de combos
(Ctrl+N/K/P/B/W, Escape) sem nenhuma customização. Este ciclo substitui
isso por um `GlobalKeymap` (mesmo padrão do `VimKeymap`, ciclo 092,
usando o modal genérico extraído no ciclo 104): uma tecla configurável
por ação, cobrindo TODAS as ações já existentes hoje + navegação por
região (foco na sidebar/editor/abas).

## Critérios de aceite

- [ ] `ui/src/state.rs`: `GlobalKeymap` — uma tecla (com modificador
      opcional, ex: `Ctrl+N`) por ação, `Default` com os binds ATUAIS
      como padrão (não muda nenhum atalho existente sem o usuário
      mexer), persistido via `gloo_storage` (mesmo padrão de
      `VimKeymap`)
- [ ] Ações cobertas nesta v1 (todas já existem como funcionalidade
      hoje, só ganham keybind configurável):
      Nova página, Nova pasta, Alternar tema, Alternar sidebar,
      Ir pra Hoje, Ver Tags, Ver Assets, Abrir paleta de comandos,
      Salvar, Fechar aba atual, Próxima aba, Aba anterior, Alternar vim
      mode, Desfazer, Refazer, **Focar sidebar**, **Focar editor**
      (as duas últimas são NOVAS — região de foco, ver ciclo 106)
- [ ] `app.rs`'s `onkeydown` vira um DISPATCHER: olha a tecla
      pressionada, procura qual ação do `GlobalKeymap` corresponde,
      dispara o callback certo — em vez de um `match` hardcoded por
      combo de tecla
- [ ] Novo item no menu ⚙ "Atalhos globais..." abre o modal de
      configuração (reaproveita o componente genérico do ciclo 104)
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

- Atalhos com sequência de 2+ teclas (estilo `g` depois `g` do vim) —
  `GlobalKeymap` v1 é só combos de 1 tecla + modificador, igual já é
  hoje
- Detectar/avisar sobre conflito entre `GlobalKeymap` e `VimKeymap`
  (ex: usuário configura a mesma tecla nos dois) — são contextos
  diferentes (global vs. dentro do editor em modo Normal), conflito é
  raro e o comportamento (o que roda primeiro, editor ou app) já é bem
  definido pela ordem de bubbling de evento existente
- Cobrir ações internas de embed (mover card do kanban só com teclado,
  etc.) — fica pro framework crescer depois, fora de escopo aqui

## Notas

Depende do ciclo 104 (modal de captura genérico). As duas ações NOVAS
("Focar sidebar"/"Focar editor") não fazem nada sozinhas ainda — ficam
prontas pro ciclo 106 (navegação por teclado na sidebar) implementar o
que "focar a sidebar" realmente significa (estado de item destacado +
navegação por seta). Adicionar essas duas ações AGORA, mesmo sem
comportamento completo ainda, evita ter que voltar no keymap depois.
