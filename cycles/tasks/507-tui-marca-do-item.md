---
id: "507"
titulo: "TUI: lista numerada mostra o número e checklist mostra a caixinha"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: []
estima_min: 30
---

# 507 — A marca de cada item

Achado ao responder "não tem como aplicar as formatações do markdown na
TUI?". Negrito, itálico, riscado, código, link e cor já saem formatados.
Mas todo item de lista saía com `-`: "1. um" aparecia como "- um", e
"- [x] feito" mostrava o `[x]` cru. As respostas do agente na conversa
eram afetadas também, porque os passos numerados delas viravam
marcadores.

## Critérios de aceite

- [x] O item de lista numerada mostra `1.`, `2.`… pela posição.
- [x] O item de checklist mostra `☐`/`☑` no lugar de `[ ]`/`[x]`, e o
      texto sem a marca, com a formatação inline.
- [x] Só desenho: o arquivo e a edição não mudam.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
```

## Não-objetivos

- Lista numerada que começa noutro número (`3.`): conta a partir de 1.
- `==marcado==` e `<span style="color:red">`: não são formatos que o app
  grava. O realce e a cor do app (`cor--x`, `fundo--x`, `color:#hex`) já
  saem.
