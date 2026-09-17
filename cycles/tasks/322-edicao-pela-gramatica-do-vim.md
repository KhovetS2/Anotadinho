---
id: "322"
titulo: "Editar embeds pela gramática do vim"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["318", "319", "320", "321"]
estima_min: 120
---

# 322 — Editar embeds pela gramática do vim

## Por quê

Do 318 ao 321 cada embed ganhou teclas soltas (`c`, `x`, `<`, `>`, `+`,
`-`) que passavam POR FORA da gramática do vim do núcleo: `c` não era
operador, `3>` não existia, `+` e `-` brigavam com os movimentos do vim.
A pessoa pediu um padrão consolidado.

## O que mudou

A gramática (`core/vim.rs`) ganhou `>>`/`<<` (operador dobrado, com
contagem) e `Ctrl+A`/`Ctrl+X` (com contagem), e a tabela `edicao_de`, que
diz o que um comando significa sobre um ITEM de embed:

| Comando | Item |
|---|---|
| `o` / `O` | cria depois / antes |
| `i` `a` `I` `A` | reescreve, com o texto atual |
| `cc` `S` `C` | reescreve do zero |
| `dd` `D` | apaga (e guarda no registro) |
| `x` | apaga o conteúdo (numa célula, o valor) |
| `yy` `Y` / `p` `P` | copia / cola depois, antes |
| `>>` `<<` | anda com o item (dias, colunas, posição) |
| `Ctrl+A` `Ctrl+X` | estica/encurta, ou gira a opção |
| `~` | alterna (recolhido, destaque) |
| `u` / `Ctrl+R` | desfaz / refaz |

A TUI passa toda tecla pela gramática e, se o comando é edição e o cursor
está num embed, manda pro módulo novo `app/edicao.rs`; senão, navegação.
Fora de embed nada muda. Desfazer guarda texto e cursor de antes de cada
edição, e é esquecido ao abrir outra página. O `main` manda `Ctrl+letra`.

Na janela, `>>`/`<<` e `Ctrl+A`/`Ctrl+X` ainda não fazem nada no texto.

## Critérios de aceite

- [x] As edições dos quatro embeds pela tabela acima
- [x] Contagem em `>>` e `Ctrl+A`
- [x] `yy`/`dd` + `p`/`P` em calendário, kanban, cronograma e tabela
- [x] `u` volta o texto exato; `Ctrl+R` refaz
- [x] A janela compila

## Comandos de validação

```bash
cargo test --workspace
(cd ui && cargo check --target wasm32-unknown-unknown)
```
