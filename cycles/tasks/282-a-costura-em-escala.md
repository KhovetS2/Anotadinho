---
id: "282"
titulo: "A costura em escala, e a bateria que ninguém rodava"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["281"]
estima_min: 120
---

# 282 — A costura em escala

## Como apareceu

Rodei `--estresse` antes de commitar o ciclo 281, porque ele mexe em
caminho quente. A bateria estava **1 de 9**, com sete "Script execution
timeout".

Guardei o 281 no stash e medi de novo: **exatamente o mesmo resultado**.
Não era do 281.

## Defeito 1 — a costura é quadrática por tecla

`alinhar` monta a tabela clássica de LCS, `O(n×m)` em tempo e memória. E
eu escrevi no comentário dela, no ciclo 276:

> "As páginas têm dezenas de unidades, então `O(n×m)` aqui é ruído"

Nunca medi isso. A página da bateria tem 1200 blocos, e `costurar`
roda no `oninput` do editor — **1,44 milhão de comparações de unidade
por caractere digitado**. O webview parava de responder.

É exatamente o que o ciclo 259 documentou e o que a bateria existe pra
pegar. Ela pegou; eu é que não a rodei no 276.

### O conserto

Aparar prefixo e sufixo iguais ANTES do LCS. Digitar muda uma unidade,
então o problema que sobra tem o tamanho da EDIÇÃO, não o da página. A
resposta é a mesma: unidade igual na mesma ponta casa consigo em
qualquer alinhamento ótimo.

Com teto no miolo: se mesmo aparado ele passa de 400×400, a página
inteira mudou, não há o que costurar, e a costura desiste — que é o
comportamento de antes dela existir.

## Defeito 2 — duas asserções impossíveis desde o ciclo 273

"todo bloco é editável" contava `[data-nav-block]`, que inclui a LISTA.
Desde o 273 a lista é grupo, `contenteditable="false"` de propósito,
porque quem digita é o item.

A asserção virou impossível naquele dia e ninguém percebeu, porque a
bateria não roda com a suíte. Passou a contar só `[data-nav-block=
"texto"]`.

## O que isto ensina sobre a bateria

Ela fica fora de `todos` por bons motivos (leva minutos, mede tempo).
O custo disso é que ela apodrece calada: dois ciclos a quebraram sem
ninguém saber. Vale rodá-la em todo ciclo que mexe no editor, e o
AGENTS.md passa a dizer isso.

## Critérios de aceite

- [x] O aparo não muda nenhuma resposta da costura
- [x] Um teste fixa a página grande com uma edição só
- [x] Um teste fixa a desistência quando tudo mudou
- [x] `--estresse` volta ao verde (9/9, era 1/9)
- [x] A suíte não muda de resultado (259/259)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --estresse
node scripts/uitest/run.mjs
```
