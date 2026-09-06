---
id: "285"
titulo: "O vim e a edição saem do navegador"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["284"]
estima_min: 180
---

# 285 — O vim e a edição saem do navegador

Os dois primeiros passos do porte pro terminal, e os dois valem por si
mesmos: são dívida que a janela paga hoje.

## Passo 1 — o vim já era do núcleo, só morava errado

`ui/src/vim_comandos.rs` tem **zero** referências ao DOM. São 558 linhas
de gramática do vim (contagem, operador, movimento, alcance
inclusivo/exclusivo tirado do Neovim no ciclo 260) com 19 testes, e um
único consumidor.

Vira `anotadinho_core::vim`. Não é refatoração: é mudança de endereço.

`vim_visual.rs` FICA na UI, e isso é a resposta certa: ele mexe em
`window.getSelection()`, que é o navegador. A gramática é do núcleo, o
realce é da tela.

## Passo 2 — a árvore aprende a mudar

A árvore sabia ler (`analisar`) e escrever (`escrever_costurando`), e
não sabia mudar. Editar era digitar no `contenteditable` e reconstruir
por `html_to_md` — 6918 linhas amarradas a um navegador.

Entram as operações do editor de texto ditas sobre a árvore: escrever,
dividir, juntar, inserir, remover, trocar tipo. Zero DOM.

## O invariante que decide se a edição chega no arquivo

`escrever_costurando` devolve pelos bytes originais quem ainda tem
`fonte`, e para de descer quando acha uma. Então editar precisa apagar a
fonte da LINHAGEM inteira, não só da unidade tocada.

Medido: com a limpeza só na folha, editar `- dois` numa lista devolve
`- dois` no arquivo. A edição some, calada.

## Critérios de aceite

- [x] O vim vive no núcleo e a janela continua passando
- [x] As operações de edição existem e são testadas sem navegador
- [x] O teste do invariante foi visto REPROVANDO pelo comportamento
- [x] `intervalo` sobrevive à edição; `fonte` não
- [x] Operação recusada não mexe na árvore
- [x] A suíte não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
node scripts/uitest/run.mjs
```
