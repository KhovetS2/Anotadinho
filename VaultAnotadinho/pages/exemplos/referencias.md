---
title: Referências — wikilink, transclusão e bloco
tags: [demo, referencia]
---
# Três jeitos de apontar pra outra página

## `[[Página]]` — link

Vira um link clicável e uma aresta no [[Grafo do Vault]]. O texto da
página alvo NÃO entra aqui: quem quiser ler, clica.

Exemplo: a [[Missão]] explica por que o Anotadinho existe.

## `![[Página]]` — transclusão

Traz o conteúdo da outra página pra dentro desta, sempre atualizado.
Nada é copiado: editar continua sendo na origem.

![[Nomenclatura]]

## `![[Página#Seção]]` — só um pedaço

Recorta do heading indicado até o próximo do mesmo nível (as
sub-seções vêm junto).

![[Guia do Agent OS#As 3 camadas]]

## `![[Página^bloco]]` — uma linha

Pra citar uma frase específica sem arrastar a seção inteira. O id nasce
sob demanda: com o bloco focado no modo de navegação, `c` grava um
`^id` naquela linha — **e só nela** — e copia a referência pronta.

O bloco abaixo vem da página de consultas, que marcou aquela linha com
`^neq`:

![[Consultas — listas que se mantêm sozinhas^neq]]

> Um bloco que ninguém referenciou continua sem marca nenhuma no
> arquivo. É por isso que o id é escrito na hora de copiar a
> referência, e não em todo bloco.

## Pelo terminal

```
anotadinho-cli --vault . read "pages/exemplos/consultas.md^neq"
```

> Transcluir um bloco da PRÓPRIA página é barrado junto com a
> auto-transclusão — a mensagem aparece no lugar, em vez de a página se
> aninhar em si mesma.
