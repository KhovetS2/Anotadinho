---
id: "473"
titulo: "Sprints — na TUI: filtro do quadro, mover de sprint e a lista de sprints"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["472"]
estima_min: 180
---

# 473 — As sprints na TUI

Quinto ciclo da proposta `pages/propostas/sprints-no-projeto.md`: a
mesma capacidade do 472, no terminal.

## Critérios de aceite

- [x] O quadro mostra o recorte do filtro, a sprint ativa por padrão
      (`sprint::tarefas_do_quadro`). As dependências olham o projeto
      INTEIRO.
- [x] Uma linha no cabeçalho diz o filtro e, na ativa, o objetivo e o
      fim.
- [x] `s` escolhe o filtro (ativa, planejadas, backlog, todas, nunca as
      encerradas). O filtro sobrevive à releitura de 3 s.
- [x] `m` move a tarefa selecionada de sprint: backlog, a ativa e as
      planejadas.
- [x] `S` abre as sprints:
  - [x] iniciar uma planejada (o erro de "já há ativa" aparece);
  - [x] encerrar a ativa em duas opções, "pendentes → próxima (nome)" e
        "pendentes → backlog": é a pergunta, feita como escolha;
  - [x] "+ Nova sprint…" pede o nome;
  - [x] as encerradas aparecem como histórico, com feitas/total.
- [x] O cursor continua seguindo a tarefa quando ela sai do recorte.
- [x] Testes de unidade, foto de tela, e pty.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
```
