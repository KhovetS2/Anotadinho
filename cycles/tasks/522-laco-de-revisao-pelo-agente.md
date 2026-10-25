---
id: "522"
titulo: "O laço de revisão: o revisor lê o diff depois da verificação e aprova ou devolve pro executor, até o teto de rodadas"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["521"]
estima_min: 240
---

# 522 — O tech lead revisa antes da PM

Com um `revisor` no projeto (521), a tarefa que passou na verificação não
vai direto pra pessoa: o revisor lê a tarefa, os critérios, a verificação
e o diff, e responde **aprovada** ou **ajustes**. Ajustes viram um
comentário na tarefa e ela volta pro executor; até `rodadas_de_revisao`
vezes. Depois disso — ou aprovada — vai pra Revisão, onde a pessoa decide.

O laço inteiro mora no `ipc` (a regra é uma só pra TUI e janela): o
revisor é só mais um trabalho que a fila devolve, com `papel: revisor`, e
o fechamento dele passa pelo mesmo `handle_fechar_tarefa_da_conversa`.

## Critérios de aceite

- [x] Verificação passou (ou não há verificação) e há revisor: a tarefa
      FICA em `executando` (a vaga do projeto continua dela — o executor
      não pega outra tarefa no mesmo branch no meio da revisão) com
      `revisao_ia: pendente`. Sem revisor, nada muda.
- [x] `handle_tarefas_a_executar` reserva a revisão pendente
      (`revisao_ia: rodando`) e a devolve como trabalho com
      `papel: revisor`, o agente revisor, o `cwd` no repositório e a
      conversa `conversas/revisao-<n>.md` (`revisao_de:` no frontmatter).
      O prompt leva a tarefa, o projeto, a arquitetura, a verificação e o
      diff desde a base (cortado num teto, dizendo que cortou).
- [x] O prompt do revisor trava o papel: não edita, julga contra os
      critérios e os combinados, termina com `VEREDITO: aprovada` ou
      `VEREDITO: ajustes` e, nos ajustes, uma seção `## Ajustes` com o que
      mudar — concreto o bastante pra um modelo menor seguir.
- [x] Fechar a revisão:
      - **aprovada** → Revisão (sistema), comentário "Revisão N/T do
        agente (nome): aprovada";
      - **ajustes** com rodadas sobrando → comentário com os ajustes,
        `rodada` +1, `revisao_ia: ajustes`, e a fila dá a vez ao executor
        (`refazendo`) — a tarefa NÃO sai de `executando`. (A primeira
        versão voltava pra `executar` com uma transição nova do sistema; o
        teste `o_sistema_nunca_decide_por_humano` reprovou, e com razão:
        `executar` é o gatilho, e é da pessoa. Ficar em `executando`
        mantém a regra sem exceção e ainda segura o branch entre as
        rodadas.)
      - **ajustes** no teto → Revisão com `revisao_ia: esgotou` e os
        ajustes no comentário: a pessoa decide;
      - sem veredito, falha ou interrompida → Revisão com
        `revisao_ia: falhou` e o motivo: o trabalho passou na verificação,
        e perder a rodada por um revisor quebrado seria pior.
- [x] A pessoa mover a tarefa pra Executar zera `rodada` e `revisao_ia`:
      um "refaça" dela começa um laço novo.
- [x] Testes de ponta a ponta no `ipc` com repositório git de verdade:
      executa → verifica → revisa (ajustes) → executa → revisa (aprovada)
      → Revisão; e o teto.

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-ipc --lib revis
python3 scripts/tui/piloto-projeto.py --revisor
```

## Não-objetivos

- A base de problemas (523) e os números por papel (524).
- Telas: o laço roda nas duas pelo caminho que já existe (o trabalho
  devolvido pela fila); mostrar a rodada no cartão é 525/526.
