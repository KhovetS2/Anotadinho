---
id: "181"
titulo: "Novo bloco pelo teclado no nav-mode"
status: done
criado: 2026-08-21
autor: humano
prioridade: media
depende_de: ["174"]
estima_min: 45
agente_alvo: claude-opus-5
---

# Novo bloco pelo teclado no nav-mode

## Objetivo

Pedido do usuário: navegando por blocos, não havia como CRIAR conteúdo
— o mouse tem o botão "+" que aparece no hover, e o teclado não tinha
equivalente. Pra inserir qualquer coisa era preciso sair do nav-mode,
achar o fim do texto e digitar.

Com um bloco focado, `n` abre um bloco novo logo abaixo e já traz o
menu `/` — de onde saem tanto os blocos de markdown quanto os 9 embeds.

## Critérios de aceite

- [x] `n` com um bloco focado cria um bloco novo abaixo dele
- [x] O menu `/` abre junto, com a lista completa (blocos + embeds)
- [x] Escolher um item insere no bloco novo, sem tocar no conteúdo que
      já existia antes ou depois
- [x] `n` digitado com o cursor NO TEXTO continua sendo a letra n
- [x] A tecla aparece no cheatsheet
- [x] Cenário no harness cobrindo o caminho inteiro até o disco

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs "'n' abre"
```

## Não-objetivos

- Criar bloco ACIMA do focado (o "+" de hover tem as duas direções; no
  teclado, criar abaixo e mover com Alt+↑ resolve o caso raro)
- Menu próprio de inserção: o `/` já é o menu de inserir do app, e ter
  dois seria manter dois

## Notas

Não mexe no markdown: põe o cursor no fim do bloco, deixa o
`contenteditable` criar o parágrafo (o mesmo que apertar Enter) e
digita "/". Daí o fluxo de sempre assume — inclusive o `select_slash`,
que já sabe inserir cada tipo sem corromper o documento (ciclos 082 e
084). Mexer no markdown na mão aqui significaria reimplementar isso.
