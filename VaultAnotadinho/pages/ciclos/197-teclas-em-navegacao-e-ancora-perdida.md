---
title: Ciclo 197 — Tecla comum em navegação, âncora perdida e trava do harness
type: ciclo
ciclo: "197"
status: concluida
date: 2026-08-21
prioridade: alta
depende_de: [194, 196]
tags:
- ciclo
---

# Ciclo 197 — Tecla comum em navegação, âncora perdida e trava do harness

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Tecla comum em navegação e âncora perdida

## Objetivo

Dois bugs relatados pelo usuário e uma falha do próprio harness achada
ao corrigi-los.

1. **Tecla comum em navegação virava texto.** Fora do modo de edição,
   qualquer letra que não fosse comando era inserida no bloco de
   markdown focado.
2. **Referência de navegação perdida.** Abrir a paleta a partir de um
   botão de ação, escolher uma página e dar Enter abria a página, mas o
   nav-mode ficava sem item — as setas paravam de andar.

## Critérios de aceite

- [x] Em navegação, tecla imprimível que não é comando é engolida.
- [x] A guarda fica DEPOIS dos comandos de bloco, senão engoliria
      `d`, `n`, `y`, `K` e `J`.
- [x] Seta com o foco perdido reancora no grupo atual, ou na raiz se o
      grupo sumiu — em vez de não fazer nada.
- [x] Reancorar só quando o foco não é de ninguém (`<body>` ou
      `.app-root`): com foco num campo ou delegate, a seta é deles.
- [x] Cenários pros dois.
- [x] O harness normaliza as configurações persistidas antes de rodar.
- [x] O `Salvar` do harness recusa gravar fora da página de rascunho.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Tirar o `contenteditable` do bloco durante a navegação: mexeria no
  foco a cada troca de modo, e `prevent_default` resolve sem tocar no DOM.

## Falhas do harness corrigidas junto

**Configuração persistida quebrando a suíte.** `nav_mode_enabled` ficou
`false` no `localStorage` (desliguei sem querer durante uma depuração
manual) e três cenários passaram a falhar sem nenhuma mudança de código
relacionada. O `run.mjs` passou a normalizar nav-mode ligado e vim
desligado antes de rodar.

**O harness gravava em página REAL.** Um cenário que navegou pra uma
página do vault e chamou Salvar reescreveu
`pages/exemplos/composicao.md`. O `SALVAR` dos quatro arquivos agora
confere o título aberto e falha alto em vez de gravar.

## Resultado

# 197 — Tecla em navegação, âncora perdida e trava do harness

## O que mudou

- `ui/src/components/editor.rs`: em navegação, tecla imprimível de um
  caractere é engolida com `prevent_default`. Posicionada DEPOIS dos
  comandos de bloco — na primeira tentativa ficou antes e engoliu
  `d`/`n`/`y`/`K`/`J`.
- `ui/src/nav_mode.rs`: `reancorar_se_perdido` — perdeu a referência
  micro, cai pra uma macro (primeiro item do grupo, ou da raiz).
- `ui/src/app.rs`: as setas tentam reancorar antes de desistir, mas só
  quando o foco não é de ninguém.
- `scripts/uitest/blocos.mjs`: 2 cenários novos; helper `bloco()` passou
  a aceitar o número do ciclo.
- `scripts/uitest/run.mjs`: normaliza as configurações persistidas.
- `SALVAR` dos 4 arquivos de cenário: trava contra gravar em página real.

## Por que `prevent_default` e não tirar o `contenteditable`

Tirar e devolver o atributo a cada troca de modo mexeria no foco duas
vezes por transição — e foco é justamente o que vinha quebrando nesta
área. `prevent_default` resolve sem tocar no DOM.

## Validação

- `cargo test --workspace`: 0 falhas; `ui`: 39 testes.
- `trunk build`: `✅ success`; Tauri: 0 erros.
- `node scripts/uitest/run.mjs`: **85/85 em 462.8s**, e desta vez com
  `git status VaultAnotadinho` limpo ao fim.
