---
id: "286"
titulo: "Passo 3: o Anotadinho num terminal, só leitura"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["285"]
estima_min: 240
---

# 286 — O Anotadinho num terminal

## O que é

O terceiro passo do porte: laço de eventos, janela que rola, desenho em
células, navegação pelos blocos. Só leitura — a edição já existe no
núcleo (ciclo 285) e entra depois.

`crates/tui`, binário `anotadinho-tui`, com ratatui e crossterm.

## Por que crate próprio e não subcomando do CLI

O `anotadinho-cli` é headless de propósito: serve script e agente, e
`ver` já imprime a estrutura de uma página pra quem quer pipe. Uma TUI é
interativa e traz duas dependências pesadas junto. Separar mantém o CLI
scriptável.

## O que é daqui e o que é do núcleo

Daqui: laço, janela, desenho, e a tradução de tecla do crossterm.

Do núcleo: a árvore, para onde o cursor vai (`navegacao::mover`), o que
cada unidade é (`Politica`) e o conteúdo dos embeds (ciclo 283).
**Nenhuma regra de navegação foi reescrita** — se tivesse sido preciso,
seria sinal de que o passo 4 não tinha terminado.

## Critérios de aceite

- [x] Abre o vault, lista as páginas, desenha uma delas
- [x] `j`/`k` andam pelos blocos por `navegacao::mover`
- [x] Enter/Escape descem e sobem de nível
- [x] A janela rola atrás do cursor, o mínimo
- [x] O conteúdo dos embeds aparece
- [x] O desenho é testado em buffer, não só na mão
- [x] Sem terminal de verdade, a mensagem explica
- [x] A suíte da janela não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
anotadinho-tui --vault <vault>
```
