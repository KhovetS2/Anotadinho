---
id: "124"
titulo: "Modal com foco automatico trap de Tab e Escape"
status: pending
criado: 2026-08-09
autor: humano
prioridade: alta
depende_de: ["123"]
estima_min: 75
agente_alvo: claude-sonnet
---

# Modal com foco automático, trap de Tab e Escape

## Objetivo

Bug reportado: abrir o diálogo "Escolher template" (`PendingDialog::
Select`, usado ao criar página) via atalho de teclado (Ctrl+N) e não
conseguir selecionar as opções pelo teclado. Causa raiz: `Modal`
(`ui/src/components/modal.rs`) — usado por Prompt/Confirm/Select/
Propriedades/Histórico — nunca move o foco pra dentro de si quando
abre, não tem handler de Escape (só fecha clicando fora), e não trapeia
Tab (some pra fora do modal). `PendingDialog::Prompt` só funciona hoje
por acaso, porque o `<input>` tem `autofocus` manual — `Select`,
`Confirm` e o resto não têm nada.

## Critérios de aceite

- [ ] `Modal` foca automaticamente o primeiro elemento focável dentro
      de si (`button`/`input`/`select`/`a`/`[tabindex]`) assim que
      `open` vira `true` — via `use_effect_with(props.open, ...)` +
      `NodeRef` no container, sem precisar que cada consumidor
      (`dialog_host.rs` etc) implemente `autofocus` manualmente
- [ ] Tab/Shift+Tab dentro do modal fica preso (cicla do último pro
      primeiro elemento focável e vice-versa) — não escapa pro resto
      da página
- [ ] Escape fecha o modal (chama `props.on_close`) de qualquer lugar
      dentro dele, não só clicando fora
- [ ] `PendingDialog::Select` (`dialog_host.rs`): setas
      cima/baixo movem entre as opções (`<li>`), Enter ativa a opção
      focada — mesmo espírito do `command_palette.rs` (que já tem essa
      navegação, usar como referência), sem precisar reimplementar do
      zero um sistema de índice ativo se der pra confiar só no foco
      nativo do navegador entre os `<button>` (Tab/Shift+Tab já
      cobrem "mover entre opções"; setas são um adicional de
      conveniência, não bloqueante)
- [ ] `autofocus` manual do `PendingDialog::Prompt` continua funcionando
      (não duplica/conflita com o autofoco novo do `Modal`)
- [ ] `cd ui && cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam
- [ ] Validação ao vivo via MCP `tauri`: abrir "Nova página" via
      atalho, confirmar que o modal "Escolher template" já nasce com
      foco em algo dentro dele (`document.activeElement` dentro do
      `.modal`), navegar as opções só com teclado (Tab/setas), Enter
      escolhe, Escape fecha sem escolher

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Reimplementar Confirm/Alert com navegação de setas — só têm 1-2
  botões, Tab already resolve
- Focus trap em popovers/menus dropdown fora do `Modal` (⚙, git
  status, ⋯ do editor) — esses não usam o componente `Modal`, são
  `<div>` próprios; ciclo 125 cuida deles separadamente
- Restaurar o foco pro elemento que abriu o modal, ao fechar — bom
  ter, mas não é o bloqueante reportado; fica pra depois se notarem falta

## Notas

`command_palette.rs` já tem o padrão de referência pronto (índice
ativo + `ArrowDown`/`ArrowUp`/`Enter`/`Escape`, ver
`command_palette.rs:74` e `:188-201`) — copiar a técnica, não
reinventar.

O foco automático do `Modal` é o item que desbloqueia TUDO que usa
`Modal` de uma vez (Prompt/Confirm/Select/Propriedades/Histórico) —
prioridade alta porque é a correção mais barata com o maior alcance
deste tema.
