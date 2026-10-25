---
id: "523"
titulo: "A base de problemas do projeto: o revisor anota o erro que se repete, a pessoa aprova, e ele entra no detalhe e na execução das próximas tarefas"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["522"]
estima_min: 150
---

# 523 — O que o júnior errou vira critério da próxima tarefa

Pedido: os erros que o revisor detecta devem formar uma base de problemas
do projeto (como o `pages/padroes` do vault), revisada pela pessoa, que
melhora o detalhamento das tarefas seguintes e a execução delas.

## Critérios de aceite

- [x] A pergunta do revisor pede, opcionalmente, uma seção `## Problemas`:
      só o erro que GENERALIZA (vale pra outras tarefas), no formato
      `- **título**: o que aconteceu — como evitar`, sem repetir o que a
      base anexada já tem.
- [x] Fechar a revisão com problemas cria uma PROPOSTA (autor `revisor`)
      pra `projetos/<slug>/problemas.md`: a página nasce na primeira vez
      (`type: problemas`), e cada problema novo é uma seção com de onde
      veio (tarefa e data). Título que já existe não entra de novo.
      Proposta pendente pra mesma página é a base da nova, pra aplicar a
      última não apagar a anterior.
- [x] Propor falhando (permissões, por exemplo) não derruba o fechamento:
      o motivo vai no comentário da revisão.
- [x] A base entra como contexto:
      - na execução e na revisão, quando a página existe;
      - no detalhamento, onde a conversa da tarefa a anexa e a pergunta
        pede critérios que cubram os problemas que se aplicam.
- [x] `projeto ver` (CLI) diz quantos problemas a base tem.

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --lib problemas
cargo test -p anotadinho-ipc --lib problema
python3 scripts/tui/piloto-projeto.py --revisor
```

## Não-objetivos

- Tela própria pra base: ela é uma página comum, revisada pela fila de
  propostas que já existe.
- Problemas propostos pelo executor: quem julga o erro é o revisor.
