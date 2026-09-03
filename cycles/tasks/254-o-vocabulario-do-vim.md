---
id: "254"
titulo: "O d do visual que não apagava, e o vocabulário do vim"
status: done
criado: 2026-09-03
autor: humano
prioridade: alta
depende_de: ["252"]
estima_min: 180
agente_alvo: claude-opus
---

# O d do visual que não apagava, e o vocabulário do vim

## O bug

Relatado ao vivo: no modo visual, selecionar e apertar `d` não mudava
nada na tela; ao salvar ou trocar de página vinha "gravação recusada:
isso apagaria as 888 letras".

A causa é `copiar_para_area_de_transferencia`. Ele cria um `<textarea>`,
chama `area.select()` — que **substitui a seleção do documento** — e o
remove em seguida. A seleção fica pendurada num nó desconectado.

O `d` do visual copia antes de apagar. Então na hora de apagar já não
havia mais o que apagar.

Quem só copiava (`c`, do ciclo 176) nunca notou: perder a seleção depois
de copiar não incomoda ninguém. Só incomoda quem ia usá-la a seguir.

## O vocabulário

Pedido junto: os atalhos gerais do vim, da folha de referência do
vim.rtorr.com.

O que impedia isso não era a lista, era a forma. Cada tecla do modo
normal era um `else if` que fazia uma coisa. Isso comporta `j` e `x`, e
não comporta `3j`, `dw` nem `d3w` — nesses a tecla não é o comando, é uma
PARTE dele.

`ui/src/vim_comandos.rs` guarda a gramática, e só ela:

```text
comando := [contagem] (operador [contagem] movimento | operador operador | ação)
```

Sem DOM, então testável de verdade — 17 testes Rust, que é o oposto do
que dava pra fazer com a cadeia de `else if`.

Entrou: contagens (`3j`, `2d3w`), `w b e 0 $ gg G`, os operadores
`d c y` com movimento e dobrados (`dd cc yy`), `D C Y S`, `x X`, `p P`,
`i a I A o O`, `r`, `J`, `~`, `u`, `Ctrl+R`.

Fica fora, como a spec já dizia: macros, registradores nomeados e `.`.

## Critérios de aceite

- [x] `d` no visual apaga o que estava selecionado
- [x] Nenhum comando deixa o arquivo sem corpo
- [x] Contagem, operador+movimento e operador dobrado funcionam
- [x] O comando pela metade aparece na barra, e Escape cancela
- [x] Cada comando que ALTERA o arquivo tem cenário que aplica, salva,
      troca de página e volta — o estrago aparecia dois passos depois do
      comando, então o cenário tem que ir até lá
- [x] O cheatsheet documenta o vocabulário novo

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --target x86_64-unknown-linux-gnu
node scripts/uitest/run.mjs
```
