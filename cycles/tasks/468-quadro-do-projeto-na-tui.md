---
id: "468"
titulo: "TUI: o projeto vira quadro com colunas, como na janela, navegado por core::espacial"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["467"]
estima_min: 180
---

# 468 — O quadro do projeto na TUI

Pedido da pessoa: "a parte do projeto necessita de melhorias visuais
para ficar semelhante à tela na GUI, e existe o kanban lá; lembre-se de
utilizar a base teórica para criar navegação".

No 461 a tela do projeto virou LISTA por etapa, porque oito colunas não
cabem num terminal. A janela resolve isso com rolagem horizontal do
quadro, e o kanban da TUI já desenha colunas. O projeto passa a ser
quadro.

## A base teórica

- **`core::espacial`:** "pra baixo" é uma pergunta sobre a TELA. O
  ciclo 268 o criou porque, num kanban, nenhuma regra estrutural acerta o
  `j`: a ordem de documento passava pelos botões, e "entre pares" andava
  entre colunas. `h`/`j`/`k`/`l` escolhem o vizinho por geometria, com o
  desvio lateral penalizado (`PESO_DESVIO`). Uma coluna vazia não tem
  candidato e é pulada sozinha.
- **`core::navegacao`:** na borda o cursor FICA, e não dá a volta nem
  muda de lugar sozinho. Abrir a tarefa (Enter) é o ato de entrar.

## Critérios de aceite

- [x] Uma coluna por etapa, com cabeçalho colorido e contagem, como
      `.projeto__coluna` da janela.
  - [x] Cartão: número, título e dependência, esta em atenção se estiver
        pendente.
  - [x] Coluna vazia aparece vazia.
- [x] As colunas que não cabem ficam fora, com "◂"/"▸" dizendo quantas.
      A janela de colunas acompanha o cursor.
- [x] Uma linha fixa sobre o quadro mostra a tarefa selecionada e as
      transições numeradas. O número move pela transição, como antes.
- [x] `h`/`j`/`k`/`l` e as setas escolhem o vizinho por
      `espacial::vizinho`. Na borda fica, e coluna vazia é pulada.
- [x] O cursor segue a TAREFA quando ela muda de coluna (o mesmo do 461).
- [x] Rolagem vertical quando a coluna é mais alta que a tela.
- [x] Testes de unidade, foto de tela regravada depois de olhar, e pty.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
ATUALIZAR_TELAS=1 cargo test -p anotadinho-tui --test telas   # depois de olhar o diff
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
```

## Não-objetivos

- Clique e arrasto no quadro da TUI (o clique ficou na sidebar, no 467)
