---
id: "251"
titulo: "Seleção e cópia atravessando blocos"
status: done
criado: 2026-09-03
autor: humano
prioridade: media
depende_de: ["250"]
estima_min: 150
agente_alvo: claude-opus
---

# Seleção e cópia atravessando blocos

## Objetivo

Fecha a spec [[Seleção e cópia atravessando blocos]] — a única pendência
que o ciclo 175 deixou de propósito. Com um `contenteditable` por bloco o
navegador não estende seleção entre blocos: arrastar de um parágrafo até
o seguinte não pega os dois, `Ctrl+A` pega só um, copiar dois parágrafos
não funciona.

## A decisão de escopo, que é o ponto todo

A spec diz que Notion e Logseq resolvem isto **reimplementando seleção do
zero** — coordenadas próprias, realce desenhado à mão, copiar/colar
interceptado — e que é trabalho grande no caminho mais usado do app.

Mas os três requisitos dela são levar, apagar e mover um CONJUNTO de
blocos. Nenhum precisa de meio bloco. E a própria spec põe seleção
parcial atravessando blocos (metade de um parágrafo até a metade do
próximo) como **não-objetivo declarado** — é ela que exigiria o motor.

Então este ciclo faz seleção por bloco INTEIRO, que atende os três RFs
sem tocar no caminho de digitação. Selecionar dentro de um bloco continua
sendo o que sempre foi: o navegador.

## Desenho

Estado no DOM (classe + atributo de âncora), como no `nav_mode` — um
estado Rust espelhando a estrutura ficaria desatualizado a cada
re-render, e aqui o re-render é constante.

A seleção vai sempre da ÂNCORA até o bloco focado. É isso que faz
encolher funcionar tão bem quanto crescer, em vez de só acumular.

Teclas, todas no modo de navegação:

| tecla | o quê |
|---|---|
| `Shift+seta` | cresce/encolhe a seleção |
| `v` | ancora e desfaz |
| seta / `hjkl` com âncora posta | continua estendendo, sem Shift |
| `Ctrl+C` | copia o conjunto como markdown |
| `d` `y` `K` `J` | agem sobre o conjunto |
| `Escape` | larga a seleção, sem sair da navegação |

`Ctrl+C` e não uma letra: `c` sozinho já copia a REFERÊNCIA de um bloco
(ciclo 176), e Ctrl+C é o gesto que já está na cabeça de todo mundo.

Com âncora posta as setas estendem sem Shift — é o que faz o modo visual
do vim (ciclo 252) cair aqui de graça, sem uma segunda implementação.

## Critérios de aceite

- [x] Selecionar três blocos e copiar produz markdown legível ao colar
- [x] Selecionar e apagar remove exatamente os blocos realçados
- [x] Mover um conjunto mantém a ordem interna, e na borda não desmancha
- [x] A seleção é visível enquanto está ativa
- [x] Seleção DENTRO de um bloco não muda — a bateria de digitação
      continua verde
- [x] Cenários de harness cobrindo seleção, cópia, apagar, mover e `v`

## Comandos de validação

```bash
cargo test --workspace
cd ui && PATH="$HOME/.cargo/bin:$PATH" cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
