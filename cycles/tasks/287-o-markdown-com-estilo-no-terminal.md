---
id: "287"
titulo: "O markdown com estilo no terminal, e a ponte coluna↔byte"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["286"]
estima_min: 180
---

# 287 — O markdown com estilo no terminal

## O que é

A TUI mostrava `**forte**` com os asteriscos, como texto plano. Aqui os
marcadores viram ESTILO: negrito é negrito, `código` ganha cor, título
fica forte, `[[wikilink]]` mostra o texto e não os colchetes.

## A consequência que decidiu a forma do ciclo

Esconder marcador muda o tamanho: `**a**` são 5 bytes no arquivo e 1
coluna na tela. As operações de edição (ciclo 285) trabalham em BYTE, e
o cursor de um editor vive em COLUNA.

Fazer só o estilo agora e a ponte depois seria retrabalho garantido: o
corte cairia no lugar errado assim que houvesse um marcador antes do
cursor. Então a ponte entra junto — `byte_da_coluna` e
`coluna_do_byte`.

## Onde mora

No NÚCLEO (`anotadinho_core::inline`), não no renderizador. Analisador
inline em dois lugares volta a divergir, como a política do bloco
divergiu até o ciclo 278. O terminal mapeia trecho → `Span`; a janela
pode mapear trecho → `<strong>` no dia em que alguém medir que os dois
discordam.

## Critérios de aceite

- [x] Marcador some da tela e vira marca
- [x] Wikilink mostra o alias e guarda o tamanho do original
- [x] Dentro de código nada é interpretado
- [x] Coluna e byte conversam nos dois sentidos, com acento
- [x] O estilo é afirmado em teste, não só o texto
- [x] A suíte da janela não muda de resultado (258/259; a reprovada
      passa isolada e foi contenção minha)

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
node scripts/uitest/run.mjs
```
