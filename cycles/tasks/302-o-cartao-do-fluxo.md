---
id: "302"
titulo: "O cartão do fluxo, cores por papel e o fundo da tela"
status: done
criado: 2026-09-07
autor: agente
prioridade: alta
depende_de: ["301"]
estima_min: 180
---

# 302 — O cartão do fluxo

## O que motivou

Um desenho de como o fluxo deveria aparecer na TUI: título
`PROPOSTA: Em Revisão`, a trilha de etapas em pílulas, a ação principal
daquele estado, e os botões de transição. Mais dois pedidos: cores
diferentes por elemento, e fundo da tela na cor da janela.

## O que é domínio e o que é desenho

Os botões de transição **não são decoração**: saem de
`Etapa::proximas()`, que é quem sabe que de "em revisão" se vai pra
aprovada, rascunho ou bloqueada — e que não se pula a revisão indo
direto pra execução. Escrever essa lista no desenho seria uma segunda
fonte da mesma verdade.

O mesmo vale pra ação principal, que vem dos ciclos 209 e 223 da janela.

Isso é derivável da própria página, diferente das linhas de uma
consulta: não se pergunta nada ao vault, só ao estado escrito no embed.

## O botão é retângulo

Depois de ver na tela, veio a correção: `[ texto ]` não é botão. Botão
tem **borda fechada dos quatro lados e superfície preenchida**, e esse
passa a ser o padrão do botão na TUI. Colchete é convenção de terminal;
num cartão cheio de texto ele não se distingue de uma citação.

Junto veio que "Pedir alteração" **também é botão** — na janela ela é
`<button>`, e no modelo era um grupo com a dica dentro. O desenho não
tinha como saber que aquilo se aperta.

Botão passa a ser **folha de fileira**, a mesma forma das transições:
quem desenha pergunta o ARRANJO, não o nome da parte.

### Célula não é pixel

O preenchimento custou duas voltas.

Primeiro pintei o fundo em todas as células do botão com a cor do papel
e deixei o traço na mesma cor: virou uma mancha maior que a caixa, sem
contorno visível.

Depois tirei o fundo do contorno, "pra cor preencher só o miolo". Sobrou
uma faixa escura entre o preenchimento e o traço — de cada lado, e em
cima e embaixo.

O erro nas duas vezes foi raciocinar em pixels. O `│` não é um risco: é
uma CÉLULA INTEIRA com um risco no meio, e o resto dela fica da cor que
estiver ali.

E aí eu escrevi que não havia terceira opção num grid de caracteres —
ou a cor toma a célula do traço, ou o vão existe. Estava errado, e a
terceira opção veio de fora: **meio-bloco**. `▄`, `▐`, `▗` pintam com o
`fg` só a metade da célula que olha PRA DENTRO do botão; a de fora fica
com o fundo da tela.

Com isso o vão some e o botão ENCOLHE meia célula de cada lado, em vez
de inchar. O respiro entre botões caiu de dois espaços pra um pelo mesmo
motivo: o contorno já deixa meia célula de fundo.

O miolo é fundo na cor do papel com texto na cor do fundo da tela — é
como `.btn--primary` se lê na janela: fundo de acento, texto que
contrasta.

## E uma regra que saiu

`j`/`k` saíam do galho numa fileira, e `h`/`l` desciam e subiam de
nível. Isso contradiz o que os ciclos 279 a 281 estabeleceram:
**movimento nunca muda de nível**. Mudar de nível é do Enter, do
Backspace e do Escape — só.

## Critérios de aceite

- [x] As transições saem do domínio, não de lista no desenho
- [x] A etapa atual se identifica pelo NOME da parte
- [x] A ação principal depende do estado e do artefato
- [x] Cada papel tem sua cor
- [x] A tela inteira tem o fundo da janela
- [x] `j`/`k` e `h`/`l` não mudam mais de nível
- [x] Botão tem contorno fechado e preenchimento sem emenda
- [x] O contorno é meio-bloco: encosta na cor sem inchar a célula
- [x] O texto do botão contrasta com o preenchimento
- [x] "Pedir alteração" é botão, não rótulo
- [x] Fileira larga quebra em grade em vez de sumir na borda
- [x] A suíte da janela não muda de resultado

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
