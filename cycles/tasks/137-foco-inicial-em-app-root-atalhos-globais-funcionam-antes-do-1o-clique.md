---
id: "137"
titulo: "Foco inicial em app-root, atalhos globais funcionam antes do 1o clique"
status: done
criado: 2026-08-09
autor: humano
prioridade: alta
depende_de: []
estima_min: 45
agente_alvo: claude-sonnet
---

# Foco inicial em app-root, atalhos globais funcionam antes do 1o clique

## Objetivo

Bug real reportado pelo usuário: `Ctrl+R` (ligar o nav-mode) não fazia
nada — nem alternava o estado, nem mostrava o badge. Causa raiz: ao
abrir o app, `document.activeElement` é `<body>` até o usuário clicar
em algo DENTRO de `.app-root`; como evento de teclado só borbulha pra
CIMA (nunca desce de volta pra um descendente), um Ctrl+tecla disparado
com o foco ainda em `<body>` nunca alcança o `onkeydown` de
`.app-root` — então NENHUM atalho global (nav-mode, paleta, Ctrl+S,
etc.) funcionava até o primeiro clique em qualquer lugar do app.

## Critérios de aceite

- [x] `app.rs`: `app_root_ref` + `use_effect_with((), ...)` focando
      `.app-root` uma vez, no mount do componente raiz — cobre o app
      inteiro (todo atalho global passa por esse `onkeydown`), não só
      o nav-mode que expôs o bug
- [x] `main.css`: `.app-root:focus-visible { outline: none; }` —
      achado durante a validação: o foco programático no mount
      ativava a heurística de `:focus-visible` do WebKitGTK, o que
      envolvia a JANELA INTEIRA num contorno azul de 2px (regra
      genérica `[tabindex]:focus-visible` do ciclo 123 não previa um
      `tabindex` na raiz do app inteiro)
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: reproduzido o bug (reload sem
      tocar em nada + Ctrl+R = nada acontece, `nav_mode_enabled`
      continua `false`); confirmado corrigido (mesmo cenário, depois
      do fix, `Ctrl+R` + seta mostra o badge); confirmado sem contorno
      visível ao redor da janela

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum — fix pequeno e cirúrgico

## Notas

### Retrospectiva sobre notas anteriores desta mesma sessão

Em ciclos anteriores (123-136), documentei repetidamente a necessidade
de chamar `document.querySelector('.app-root').focus()` antes de
testar QUALQUER atalho via MCP como sendo "requisito de configuração
do teste" ou implicitamente um detalhe só do ambiente de automação.
Essa caracterização estava ERRADA — não é uma peculiaridade do driver
MCP, é o comportamento real e correto do DOM (bubbling só sobe), e o
app de verdade tinha exatamente esse mesmo problema pra um usuário
real que abre o app e tenta um atalho ANTES de clicar em qualquer
coisa. Só não tinha sido notado porque em uso normal o usuário quase
sempre clica em algo (uma página na sidebar, o editor) logo de cara,
o que incidentalmente já resolvia o foco. O relato do usuário expôs
isso de vez.

### Por que só agora, com o nav-mode

Os outros atalhos globais (Ctrl+K, Ctrl+S, etc.) têm o MESMO problema
em teoria, mas o usuário só notou com `Ctrl+R` porque provavelmente
foi literalmente a PRIMEIRA tecla que tentou depois de abrir o app —
os outros atalhos ele deve ter usado depois de já ter clicado em algo
(abrir uma página, por exemplo), então o bug nunca apareceu pra eles
na prática. O fix cobre TODOS os atalhos globais de uma vez, não é
específico do nav-mode.
