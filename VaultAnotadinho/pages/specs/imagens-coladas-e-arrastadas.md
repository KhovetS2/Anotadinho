---
title: "Imagens arrastadas: persistir como as coladas"
type: spec
date: 2026-08-23
status: em-revisao
prioridade: alta
tags:
- spec
- editor
---
# Imagens arrastadas: persistir como as coladas
{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao

{{ /fluxo }}
## Contexto

Metade disso já existe, e a outra metade parece existir mas não.

**Colar funciona desde o ciclo 118(Não está funcionando).** `Ctrl+V` com uma imagem na área
de transferência grava o arquivo em `assets/` e insere a referência.
Só intercepta quando acha imagem, então colar texto segue intacto.(Não está funcionando corretamente verificar com imagem real se isso está realmente funcionando)

**Arrastar é uma armadilha.** Soltar um arquivo de imagem no editor
insere um `<img>` apontando pra uma URL `blob:` — um endereço que só
vale enquanto a janela estiver aberta. A imagem APARECE, a pessoa segue
trabalhando, e nada é gravado no acervo.  
**Arrastar a imagem para  uma inserção de imagem via /imagem seria importante. (Nova demanda).**

Pior: o `blob:` chega ao ARQUIVO. O `.md` fica com uma referência a um
endereço que morreu junto com a sessão, e reabrir a página mostra uma
imagem quebrada. Isso foi confirmado no harness, não deduzido.

Dois detalhes que agravam:

- O arraste insere via `execCommand("insertHTML")`, que o projeto
abandonou justamente por corromper o DOM do editor.
- Ele não marca o documento como editado, então o que ficou na tela
pode nem chegar ao arquivo.

## Requisitos funcionais

- **RF1.** Arrastar um arquivo de imagem pra dentro da nota grava a
imagem no acervo do vault e insere uma referência durável — o mesmo
resultado de colar.
- **RF2.** Reabrir a página mostra a imagem, carregada de verdade.
- **RF3.** Soltar vários arquivos de uma vez insere todos.
- **RF4.** Um arquivo que não é imagem é ignorado sem quebrar a nota.
- **RF5.** O alvo de soltar é visível durante o arraste.
- **RF6.** Se a gravação falhar, a nota não fica com referência
quebrada e o erro é dito.
- **RF7.** O nome gravado não colide com um existente nem sobrescreve
nada.
- **RF8.** Desfazer logo após inserir remove a referência da nota.
- **RF9.** Colar continua funcionando como hoje, inclusive dentro do
editor por bloco.

## Requisitos não funcionais

- **RNF1.** O `.md` continua legível fora do app: a referência é
markdown comum apontando pro acervo.
- **RNF2.** Colar TEXTO continua intacto — é o caminho mais usado do
editor.
- **RNF3.** Nada de `execCommand` pra inserir: é regra do projeto e é a
origem de bugs de DOM já pagos.
- **RNF4.** Nada sai da máquina: a gravação é local.
- **RNF5.** Soltar uma imagem grande não trava a janela.

## Critérios de aceite

- [ ] Arrastar um `.png` grava o arquivo no acervo e insere a
referência.
- [ ] Fechar e reabrir a página mostra a imagem.
- [ ] O `.md` aberto num editor comum mostra uma referência válida —
nenhum `blob:` chega ao arquivo.
- [ ] Arrastar duas vezes a mesma imagem não sobrescreve a primeira.
- [ ] Soltar um `.txt` não altera a nota.
- [ ] Colar imagem e colar texto continuam verdes na bateria.
- [ ] Cenários de harness pra arrastar e pra colar.

## Fora de escopo

- Editar a imagem (recortar, redimensionar, anotar).
- Arrastar vídeo, áudio ou PDF.
- Baixar imagem de uma URL colada.

## Perguntas em aberto

- Uma imagem inserida duas vezes deve virar dois arquivos ou reusar o
primeiro? Reusar economiza espaço mas acopla notas. Fica pra
proposta.

## Relacionado

- [[Assets]]
- [[Ciclo 118 — Colar imagem da área de transferência]]
