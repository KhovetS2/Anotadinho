---
title: 'Imagens arrastadas: persistir como as coladas'
tags:
- spec
- editor
type: spec
date: 2026-08-23
prioridade: alta
status: concluida
---
# Imagens arrastadas: persistir como as coladas
{{ type: "fluxo" }}
artefato: spec
etapa: concluida

{{ /fluxo }}
## Contexto

Metade disso já existe, e a outra metade parece existir mas não.

**Colar foi implementado no ciclo 118, mas não está funcionando
corretamente.** `Ctrl+V` com uma imagem na área de transferência deveria
gravar o arquivo em `assets/` e inserir a referência. Esse comportamento
precisa ser verificado com uma imagem real. A interceptação deve acontecer
somente quando houver imagem, mantendo intacto o fluxo de colar texto.

**Arrastar é uma armadilha.** Soltar um arquivo de imagem no editor
insere um `<img>` apontando pra uma URL `blob:` — um endereço que só
vale enquanto a janela estiver aberta. A imagem APARECE, a pessoa segue
trabalhando, e nada é gravado no acervo.

Também é desejado relacionar o arraste ao fluxo de inserção do comando
`/imagem`, mas o comportamento exato ainda precisa ser definido.

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
resultado esperado ao colar.
- **RF2.** Reabrir a página mostra a imagem, carregada de verdade.
- **RF3.** Soltar vários arquivos de uma vez insere todos.
- **RF4.** Um arquivo que não é imagem é ignorado sem quebrar a nota.
- **RF5.** O alvo de soltar é visível durante o arraste.
- **RF6.** Se a gravação falhar, a nota não fica com referência
quebrada e o erro é dito.
- **RF7.** O nome gravado não colide com um existente nem sobrescreve
nada.
- **RF8.** Desfazer logo após inserir remove a referência da nota.
- **RF9.** Colar imagem funciona corretamente, inclusive dentro do
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

- [x] Arrastar um `.png` grava o arquivo no acervo e insere a
referência.
- [x] Fechar e reabrir a página mostra a imagem.
- [x] O `.md` aberto num editor comum mostra uma referência válida —
nenhum `blob:` chega ao arquivo.
- [x] Arrastar duas vezes a mesma imagem não sobrescreve a primeira.
- [x] Soltar um `.txt` não altera a nota.
- [x] Colar uma imagem real grava o arquivo no acervo e insere uma
referência durável, inclusive dentro do editor por bloco.
- [x] Colar texto continua intacto.
- [x] Cenários de harness pra arrastar e pra colar, incluindo colar uma
imagem real.

## Fora de escopo

- Editar a imagem (recortar, redimensionar, anotar).
- Arrastar vídeo, áudio ou PDF.
- Baixar imagem de uma URL colada.

## Perguntas em aberto — respondidas

- **Uma imagem inserida duas vezes vira dois arquivos ou reusa o
primeiro?** Dois arquivos, a não ser que se passe a mesma referência.
`save_asset_bytes` numera até achar um nome livre e nunca sobrescreve.
- **O arraste insere na hora ou abre a inserção com personalizações?**
Abre o modal de personalização (alinhamento, tamanho, legenda, texto
alternativo), implementado no ciclo 242. Isso diverge da letra do RF1
("insere uma referência durável" — a inserção passou a exigir uma
confirmação), e a divergência é deliberada: foi a resposta dada aqui
mesmo, e é o que separa arrastar de colar.

## Duas notas sobre o que foi entregue

**A referência gravada é `<figure class="inserted-image">`, não
`![](…)`.** O RNF1 pede um `.md` legível fora do app apontando pro
acervo, e a `<figure>` é isso — markdown com HTML inline, que qualquer
leitor renderiza. O que ela tem a mais é onde guardar alinhamento,
tamanho, proporção e legenda, que a sintaxe curta não tem. Essa escolha
é do ciclo 226, posterior a esta spec.

**Os cenários de harness ficaram velhos antes de rodar.** Foram escritos
da letra do RF1 (gravação imediata) e da leitura mais estreita do RNF1
(`![](…)`), e reprovavam um app que já fazia o combinado por outro
caminho. Reescritos no ciclo 257 contra o critério de aceite de verdade
— referência válida, nenhum `blob:` no arquivo — e movidos pra bateria
permanente.

## Relacionado

- [[Assets]]
- [[Ciclo 118 — Colar imagem da área de transferência]]
