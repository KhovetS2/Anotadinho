---
id: "212"
titulo: "Lista numerada desalinhada do bloco"
status: done
criado: 2026-08-23
autor: humano
prioridade: media
depende_de: ["211"]
estima_min: 30
agente_alvo: claude-opus-5
---

# Lista numerada desalinhada do bloco

## Objetivo

Numa lista numerada, o marcador (`1.`, `2.`) sai da coluna de texto.
Acontece nos dois lugares onde há markdown renderizado, por motivos
diferentes.

## Critérios de aceite

- [x] No editor, o bloco de lista começa no mesmo x dos demais blocos
- [x] A linha de realce do bloco fica na canaleta, não sobre o marcador
- [x] Na conversa, o marcador fica dentro da coluna de texto da mensagem
- [x] Lista aninhada (dentro de um `li`) não é afetada

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Notas

Medido no app, não deduzido do CSS.

**Editor.** `.editor__wysiwyg ul, ol` (especificidade 0,1,1) é mais
específico que `.editor__bloco` (0,1,0), e o `margin: 0 0 0.5rem` dele
zerava a `margin-left: -0.5rem` do bloco. Resultado medido: parágrafo e
título com a caixa em x=276, lista em x=284 — a lista era o único bloco
deslocado, então o realce de borda caía dentro da coluna de texto,
encostando no marcador.

Correção: `ul.editor__bloco`/`ol.editor__bloco` recuperam a margem
negativa e o recuo do marcador vira padding interno (2rem = 0.5rem da
caixa + 1.5rem pro marcador). Depois: todos os blocos em x=276.

**Conversa.** O reset zera o padding de toda lista, e marcador é
desenhado FORA da caixa (`list-style-position: outside`). Medido: `li`
em 857, o mesmo x do parágrafo — ou seja, o "1." ficava pendurado à
esquerda, fora da coluna. Com `padding-left: 1.5rem`, `li` vai pra 881
e o marcador cabe dentro.
