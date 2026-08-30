---
id: "246"
titulo: "Arrastar imagem do sistema ainda não insere"
status: pending
criado: 2026-08-30
autor: agente
prioridade: alta
depende_de: ["245"]
estima_min: 90
---

# 246 — O arrasto do sistema ainda não insere

## Estado

O ciclo 245 consertou **metade**: soltar fora do editor não derruba mais o
app. Mas arrastar uma imagem de fora e soltar dentro do editor continua
não inserindo nada, confirmado no uso real depois da correção.

O cenário sintético equivalente **passa** — ele monta um `drop` com
`text/uri-list` apontando pra um arquivo real e vai até o asset gravado.
Então o que falha está entre o gesto real e o ponto onde o cenário começa.

## O que já se sabe

Uma sonda na janela de verdade, durante um arrasto real, registrou:

```
tipo:     drop
alvo:     editor__wysiwyg
tipos:    ["text/uri-list", "text/html"]
itens:    string:text/uri-list, string:text/html
arquivos: 0
```

Ou seja: o evento **chega no elemento certo** e anuncia ter `text/uri-list`.

## Hipótese principal

`dataTransfer.getData("text/uri-list")` devolve string vazia no drop
real, mesmo com o tipo anunciado. O WebKitGTK entrega o conteúdo só por
`items[i].getAsString(cb)`, que é **assíncrono** — e o código atual lê por
`getData`, síncrono. No evento sintético `getData` funciona, porque quem
pôs o dado ali foi o próprio JS; num arrasto do sistema, não.

Isso explicaria por que o cenário passa e o gesto não.

## Como confirmar

Instalar uma sonda na janela aberta e arrastar de novo, registrando:

- `dt.getData("text/uri-list")` — string vazia confirma a hipótese
- o resultado de `dt.items[0].getAsString(s => ...)` — se vier o caminho
  por aqui, a correção é trocar a leitura por essa via
- `e.defaultPrevented` no `dragover` imediatamente anterior

## Se a hipótese se confirmar

Ler por `getAsString` e seguir o fluxo já existente (`ler_imagens_locais`
no backend, que funciona). O caminho do backend está provado pelo cenário;
o que muda é só de onde a string vem.

## Não-objetivos

- Voltar `dragDropEnabled` pra `true` (aí o Tauri engole o evento e nem
  isso se pode investigar)
