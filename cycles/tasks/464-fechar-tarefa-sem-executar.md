---
id: "464"
titulo: "Módulo Projeto — a pessoa fecha uma tarefa resolvida sem execução"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: ["463"]
estima_min: 60
---

# 464 — Fechar uma tarefa sem executar

## O que apareceu (tarefa 001 do projeto Anotadinho)

A 001 era uma DECISÃO. O agente a executou e parou como devia
(`bloqueada`, apontando que a escolha era da pessoa). A pessoa decidiu
na conversa, a decisão foi registrada, e... não havia como fechar.
"Feita" só aceita tarefa vinda de "Revisão", e só o app leva uma tarefa
até a Revisão, depois de uma execução. Resolver sem executar não tinha
caminho: a saída era gastar uma execução (US$ 0,31 na primeira) só pra
confirmar o que já estava escrito.

## Critérios de aceite

- [x] A pessoa leva uma tarefa de `pronta` ou de `bloqueada` direto pra
      `feita`.
- [x] `feita` continua sendo só da pessoa. O sistema nunca chega lá
      (vale a varredura `o_sistema_nunca_decide_por_humano`).
- [x] A nova transição vem POR ÚLTIMO na lista, pra os números da TUI e
      a ordem dos botões que já existiam não mudarem.
- [x] Janela e TUI mostram o botão sem código próprio, porque as duas
      leem `proximas_para(Humano)`.
- [x] O guia de projetos diz quando usar.

## Decisões

- **`ideia` e `detalhando` não fecham direto.** Fechar sem executar é
  pra o que foi DECIDIDO ou resolvido por fora. Uma tarefa que nem foi
  detalhada e já "está feita" é quase sempre uma que deveria ser
  apagada, e não marcada como feita.

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs "projeto:"
```
