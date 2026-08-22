---
title: Ciclo 138 — Rede de foco geral e troca da tecla do nav-mode
type: ciclo
ciclo: "138"
status: concluida
date: 2026-08-09
prioridade: alta
depende_de: ["137"]
tags:
- ciclo
---

# Ciclo 138 — Rede de foco geral e troca da tecla do nav-mode

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Rede de foco geral e troca da tecla do nav-mode

## Objetivo

Dois bugs reais reportados pelo usuário na mesma mensagem: (1) o
mesmo bug de foco do ciclo 137 (nenhum atalho global funciona) acontece
de novo TODA VEZ que um overlay fecha via Escape, não só no boot do
app — o fix do ciclo 137 só cobria a montagem inicial; (2) `Ctrl+J`
(era `Ctrl+R`) do nav-mode não funcionava nem depois do fix do ciclo
137, quase certamente porque `Ctrl+R` colide com o atalho nativo de
"recarregar" do próprio WebKitGTK, capturado antes de chegar no JS da
página.

## Critérios de aceite

- [x] `app.rs`: rede de segurança geral via polling leve
      (`gloo_timers::callback::Interval`, 300ms) — checa se
      `document.activeElement` é `<body>` e, se for, refoca
      `.app-root`. Cobre QUALQUER overlay que feche sem devolver o
      foco (paleta, modais de diálogo, cheatsheet, configurações de
      keymap, modais locais do editor), sem precisar tratar cada um
      individualmente
- [x] `state.rs`: `toggle_nav_mode` trocado de `"r"` pra `"j"` — letra
      livre sem convenção conhecida de nível de engine/edição de
      texto (ver Notas), teste do default atualizado
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: reproduzido o bug de foco
      (Ctrl+K → Escape → foco cai em `<body>`, atalho seguinte não
      funciona); confirmado corrigido (mesmo cenário, foco volta pra
      `.app-root` sozinho, `Ctrl+J` funciona logo em seguida);
      confirmado que `Ctrl+J` funciona com o app recém-aberto, sem
      nenhuma interação prévia

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum novo — os dois fixes são extensões diretas do ciclo 137

## Notas

### Por que `focusout` não funcionou (tentativa descartada)

Primeira tentativa: um listener de `focusout` na `window`, mais
"reativo" que polling (só roda quando algo de fato perde o foco).
Não funcionou de forma confiável ao vivo — quando o elemento com foco
é REMOVIDO do DOM (o caso de toda vez que um overlay desmonta, em vez
de perder foco por um Tab/clique normal), motores diferem em disparar
`focusout` ou não; o WebKitGTK usado aqui aparentemente não dispara
nesse caso específico (confirmado testando: o foco ficava preso em
`<body>` mesmo depois de esperar). Troquei pra um polling leve
(`Interval` de 300ms, mesmo padrão já usado em outro lugar desta
função) — menos "elegante", mas resiliente contra qualquer jeito
NOVO de um overlay futuro perder o foco sem precisar lembrar de tratar
caso a caso.

### Por que `Ctrl+R` falhava mesmo com o app-root focado

`Ctrl+R` = "recarregar" é uma convenção bem mais profunda que "chrome
de navegador" (que uma WebView sem chrome de fato não tem, como já
confirmado nesta sessão pro caso do Ctrl+N) — é frequentemente
implementada como um atalho de nível de MOTOR (WebKitGTK), capturado
antes até do JS da página rodar. Já Ctrl+N parece ser mais
especificamente "chrome de navegador" (nova janela/aba), ausente numa
WebView pura — daí ter funcionado bem em testes anteriores desta
sessão. A letra nova (`j`) não tem convenção conhecida nem de motor
nem de edição de texto nativa (cut/copy/paste/select-all/bold/
italic/underline/undo/redo) — ver comentário completo em
`state.rs::impl Default for GlobalKeymap`.

## Resultado

# Ciclo 138 - done

## Resumo

Dois bugs reais reportados pelo usuário: o bug de foco do ciclo 137
acontecia de novo toda vez que um overlay fechava (não só no boot),
corrigido com um polling leve que refoca `.app-root` sempre que o
foco cai em `<body>`, sem precisar tratar cada overlay individualmente.
E `Ctrl+R` do nav-mode não funcionava mesmo com o foco correto —
trocado pra `Ctrl+J`, já que "R" de "reload" é provavelmente uma
convenção de nível de engine do WebKitGTK.

## Arquivos criados/modificados

- `ui/src/app.rs` — polling de recuperação de foco (`Interval` 300ms)
- `ui/src/state.rs` — `toggle_nav_mode` de "r" pra "j" + teste
  atualizado

## Testes

`cd ui && cargo test --lib`: 84. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: reproduzido o bug (paleta aberta
via Ctrl+K, Escape, foco cai em `<body>`, atalho seguinte não
funciona) e confirmado corrigido (mesmo cenário, recuperação
automática, `Ctrl+J` funciona logo depois); `Ctrl+J` também confirmado
funcionando com o app recém-aberto sem interação prévia.

## Notas

Tentativa inicial com `focusout` + `window` foi descartada por não
disparar de forma confiável quando o elemento focado é removido do
DOM (comportamento inconsistente entre motores) — trocado por polling,
mais simples e robusto. Ver Notas completas no arquivo de task pra o
raciocínio sobre por que "r" especificamente colidia (convenção de
"reload" em nível de motor WebKitGTK, diferente do Ctrl+N que é só
convenção de chrome de navegador, ausente numa WebView pura).
