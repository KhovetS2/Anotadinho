---
id: "129"
titulo: "Auditoria final de paridade de teclado e cheatsheet"
status: pending
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["123", "124", "125", "126", "127", "128"]
estima_min: 75
agente_alvo: claude-sonnet
---

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

- [ ] Roteiro de fluxos completos, testados ao vivo via MCP `tauri`
      SEM NENHUM clique de mouse (só teclado) do início ao fim de cada
      um:
      1. Abrir vault → criar página nova de um tipo específico (via
         paleta, ciclo 128) → editar propriedades (via painel, agora
         com foco automático do ciclo 124) → salvar
      2. Navegar pra uma página existente via sidebar (setas, ciclo
         106) → abrir o grafo → tabular até um nó → Enter pra abrir
      3. Abrir um kanban → tabular até um card → Enter → editar →
         fechar
      4. Abrir a paleta de comandos → buscar e navegar resultados →
         Enter
- [ ] Qualquer atalho/padrão de interação novo introduzido nos ciclos
      123-128 (Tab-trap em modal, setas nos menus dropdown, Enter em
      cards, etc) documentado na cheatsheet (`cheatsheet_modal.rs`) se
      fizer sentido como "atalho" (padrões de navegação genéricos como
      "Tab" não precisam de entrada própria, mas teclas específicas
      tipo "Enter abre o nó focado no grafo" sim)
- [ ] Nenhuma regressão nos testes existentes
      (`cargo test --workspace`, `cd ui && cargo test --lib`)
- [ ] Relatório final (na seção Notas deste arquivo de task, depois de
      `status: done`) listando: o que ficou 100% operável por teclado,
      e o que CONSCIENTEMENTE ficou de fora (com justificativa) —
      honestidade sobre o estado real, não alegar cobertura total se
      não for o caso

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
