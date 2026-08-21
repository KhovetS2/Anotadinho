---
id: "197"
titulo: "Tecla comum em navegação, âncora perdida e trava do harness"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [194, 196]
estima_min: 120
agente_alvo: claude-opus
---

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
