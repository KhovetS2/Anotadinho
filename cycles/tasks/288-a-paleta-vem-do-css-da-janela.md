---
id: "288"
titulo: "A paleta da TUI vem do CSS da janela"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["287"]
estima_min: 120
---

# 288 — A paleta vem do CSS da janela

## A pergunta que motivou

"Dá pra pegar o HTML+CSS e fazer a TUI parecer o app?"

Renderizar CSS num terminal, não: grade de células não tem box model,
`padding: 12px` não significa nada ali, e o `main.css` tem 7.313 linhas
de grid, flexbox e transição.

Mas metade do CSS não é geometria. São **40 variáveis de design** e
**quatro temas** — e isso É a identidade visual. Cor porta inteira.

## O que fazer

1. Ler os blocos `:root` do `main.css` em tempo de compilação e montar a
   paleta dos quatro temas;
2. Trocar as cores fixas do desenho por NOMES (`Realce::Titulo(1)`,
   `Realce::Wikilink`), como highlight group do Neovim;
3. `--tema` na linha de comando.

O item 2 é o mesmo que eu já tinha apontado como necessário pra uma TUI
personalizável. Fazer os dois juntos é o barato: eu ia mexer nessas
linhas de qualquer jeito.

## Critérios de aceite

- [x] A paleta sai do CSS, não de uma cópia
- [x] Comentário com hex dentro não vira token
- [x] `var(--x, #HEX)` é lido pelo fallback
- [x] O que não é cor fica de fora
- [x] Os quatro temas existem e são diferentes entre si
- [x] Nenhum realce cai em cor de socorro
- [x] Um teste prova que trocar o tema muda o desenho

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo clippy -p anotadinho-tui --all-targets
anotadinho-tui --vault <vault> --tema papel
```
