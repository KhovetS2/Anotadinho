---
id: "250"
titulo: "A pilha de navegação depois de abrir uma página"
status: done
criado: 2026-09-03
autor: humano
prioridade: alta
depende_de: []
estima_min: 90
agente_alvo: claude-opus
---

# A pilha de navegação depois de abrir uma página

## Objetivo

Primeira parte da spec [[Navegação por teclado consistente e modo vim
completo]] — a que é bug, não funcionalidade nova. RF1, RF2 e RF3.

Caminho relatado: home → "Trabalho recente" → Enter num card → a página
abre, e a partir daí as setas ficam presas na barra superior. Escape
devolve as setas ao editor, mas um segundo Escape não volta pra
navegação entre seções: só Backspace faz isso.

## O que estava errado

A hipótese da spec estava certa, e o próprio código já a tinha escrito.
`nav_mode::reancorar_se_perdido` diz, em comentário:

> Grupo sumiu junto (a página mudou): tenta a raiz, que é o nível mais
> macro que sempre existe.

Abrir uma página de dentro de um grupo troca o conteúdo inteiro, mas a
pilha continuava apontando pro grupo antigo. A seta seguinte não achava
o grupo, caía no resgate, e o resgate pousa na raiz — cujo primeiro item
é a barra superior.

O segundo Escape tinha duas causas somadas:

1. `app.rs` limpava a pilha INTEIRA num Escape só, de qualquer
   profundidade (`nav_stack.set(Vec::new())`), em vez de subir um nível;
2. `editor.rs:1863` tratava Escape sem checar `em_navegacao` — já
   estando em navegação, `bloco_do_cursor()` ainda achava um bloco (a
   seleção de texto continua onde estava), dava `stop_propagation` e a
   tecla nunca chegava no `app.rs`. Só Backspace subia, que é
   exatamente o que foi relatado.

O Enter já tinha essa guarda desde o ciclo 195, com comentário
explicando o "dois editores ao mesmo tempo". O Escape ficou de fora.

## Critérios de aceite

- [x] Abrir uma página de dentro de um grupo deixa o teclado nos BLOCOS
      da página aberta, não na barra superior
- [x] Escape sobe um nível por vez, de qualquer profundidade; na raiz,
      encerra a sessão
- [x] `hjkl` movem onde as setas movem (minúsculas só — `J`/`K` já movem
      o bloco, que é outra ação)
- [x] Cenários de harness para os três

## Fora deste ciclo

RF4–RF7 (modos visual e visual em bloco, `/` como comando, atalho
dedicado de navegação, modo do vim na barra) dependem de seleção
atravessando blocos, que é a spec seguinte. A própria spec diz que as
duas precisam ser planejadas juntas.

## Comandos de validação

```bash
cargo test --workspace
cd ui && PATH="$HOME/.cargo/bin:$PATH" cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
