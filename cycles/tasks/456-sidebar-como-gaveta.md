---
id: "456"
titulo: "TUI: a sidebar é uma gaveta como a da janela, e passar por cima não abre página"
status: done
criado: 2026-10-07
autor: agente
prioridade: alta
depende_de: ["454"]
estima_min: 150
---

# 456 — A sidebar como gaveta, e o cursor da lista separado da página aberta

Pedido direto da pessoa: "que na TUI a barra lateral seja como um drawer
como é na GUI" e "que ela pare de ficar carregando a página ao passar por
cima na seleção sem que tenha sido realmente selecionado".

## O que a janela faz (medido em 2026-10-07)

`.app-sidebar` tem **260px fixos**, mesmo com a janela em 1966px, e
recolhe numa trilha de 40px (`Ctrl+B`). O item da página aberta é
marcado à parte do que está sob o mouse. A TUI usava 30% da largura do
terminal, e num terminal largo a lista ocupava um terço da tela.

## A causa do "carrega ao passar por cima"

`e.pagina` fazia dois papéis: o item sob o cursor da lista **e** a
página aberta. Andar na lista (`j`/`k`, a roda do mouse, digitar na
busca) trocava `e.pagina`. O vigia do disco (`vigiar_disco`, a cada
~2 s) lê `paginas[e.pagina]` do disco, via texto diferente do da tela e
chamava `recarregar_do_disco`: a página passada por cima substituía a
aberta sem nenhum Enter. O título trocava na hora. Uma edição feita
depois disso iria pro arquivo errado.

## Critérios de aceite

- [x] Andar na lista (teclado, roda, busca) não muda a página aberta,
      nem o título, nem o que o vigia do disco relê
- [x] Enter abre o item sob o cursor
- [x] `m`, `dd` e `o` na lista agem sobre o item sob o cursor; pela barra
      de comandos, sobre a página aberta
- [x] A página aberta tem uma marca própria na lista, diferente do cursor
- [x] A sidebar tem largura fixa equivalente à da janela (32 colunas),
      sem passar de 40% de um terminal estreito, e continua recolhendo
      em trilha com `Ctrl+B`
- [x] Testes que reprovam com o comportamento antigo
- [x] Pty: passar por cima de outra página por mais de 2 s não a abre

## Interpretação (aponto a ambiguidade)

"Drawer como na GUI" foi lido como a sidebar da janela é hoje: largura
fixa, recolhível, não sobreposta. Se a ideia era uma gaveta que se abre
POR CIMA do conteúdo e fecha ao escolher, isso é outra coisa, e a janela
também não faz.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
```

## Não-objetivos

- Mudar a sidebar da janela
