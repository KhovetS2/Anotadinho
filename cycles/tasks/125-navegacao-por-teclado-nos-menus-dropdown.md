---
id: "125"
titulo: "Navegacao por teclado nos menus dropdown"
status: pending
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["123"]
estima_min: 75
agente_alvo: claude-sonnet
---

# Navegação por teclado nos menus dropdown (⚙, git status, ⋯ do editor)

## Objetivo

Auditoria encontrou 3 menus dropdown próprios (não usam `Modal`, são
`<div>` popover com fechar-ao-clicar-fora + Escape já implementados):
menu "⚙" do app (`header_bar.rs:233-298`), popover de git status
(`header_bar.rs:198-225`), e menu "⋯" do editor (`editor.rs:1642-1677`).
Todos já fecham com Escape/clique fora, mas nenhum foca o primeiro
item ao abrir nem tem navegação por seta — usuário precisa saber Tab
às cegas pra alcançar os itens.

## Critérios de aceite

- [ ] Os 3 menus (⚙, git status, ⋯ do editor) focam automaticamente o
      primeiro item (`<button>`) assim que abrem — mesma técnica do
      `command_palette.rs` (`NodeRef` + `use_effect_with` no estado
      de "aberto")
- [ ] Seta pra baixo/cima move entre os itens do menu (wrap-around,
      igual `command_palette.rs:188-201`); Enter ativa o item focado
      (já funciona via clique nativo do `<button>`, só falta a
      navegação de seta chegar até ele)
- [ ] Escape e clique-fora continuam fechando (comportamento já
      existente, não regredir)
- [ ] `cd ui && cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam
- [ ] Validação ao vivo via MCP `tauri`: abrir cada um dos 3 menus,
      confirmar foco automático no primeiro item e navegação por seta
      funcionando nos 3

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Extrair um componente `DropdownMenu` genérico reaproveitável pelos
  3 — três blocos de estado+efeito quase idênticos é mais duplicação
  do que o projeto costuma tolerar, mas os 3 menus têm formas
  ligeiramente diferentes (a popover de git tem botões de ação no
  fim, o ⋯ do editor tem um separador); decidir na hora da
  implementação se compensa extrair ou se é "3 cópias parecidas mas
  cada uma pequena o bastante pra não valer a abstração" — não travar
  o ciclo nessa decisão, só documentar a escolha feita
- Atalho de teclado dedicado pra ABRIR cada menu (ex: uma tecla só pra
  abrir o menu ⚙) — os botões que abrem já são focáveis/Tab-áveis
  (ciclo 123 já dá o indicador visual); atalho dedicado é ciclo
  futuro se pedirem

## Notas

Reaproveita a técnica já validada em `command_palette.rs` — não é
território novo, é replicar um padrão que já funciona bem em três
lugares que ainda não o usam.
