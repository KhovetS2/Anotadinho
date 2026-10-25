---
id: "524"
titulo: "Números por papel: quanto o executor e o revisor rodaram e custaram, e quantas rodadas a revisão levou"
status: done
criado: 2026-10-25
autor: agente
prioridade: media
depende_de: ["522"]
estima_min: 120
---

# 524 — Vale a pena o júnior grátis?

O arranjo PM / tech lead / júnior só se justifica se o revisor caro
custar menos do que o executor caro custaria, e se a revisão convergir.
Sem número, isso é palpite.

## Critérios de aceite

- [x] O uso do aider entra no registro: no formato texto, a linha
      `Tokens: 1.2k sent, 65 received. Cost: $0.01 message, …` vira `Uso`
      (entrada, saída e o custo da mensagem quando há). Medido na saída
      real do aider 0.86.2 (`Tokens: 813 sent, 65 received.`).
- [x] `core::numeros::do_projeto(execuções, tarefas, slug)`: por papel
      (executor = conversa `tarefa-NNN`, revisor = `revisao-NNN`,
      conversas = o resto do projeto) — execuções, falhas, tempo, tokens,
      custo conhecido e os agentes que rodaram; e da revisão — tarefas
      revisadas, aprovadas de primeira, média de revisões por tarefa,
      quantas esgotaram o teto.
- [x] `anotadinho-cli projeto numeros <slug>` (e `--json`).

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --lib numeros
python3 scripts/tui/piloto-projeto.py --revisor
```

## Não-objetivos

- Preço de modelo local: o aider não diz custo do Ollama, e inventar um
  número seria pior que dizer "sem custo informado".
- A tela: 525 (TUI) e 526 (janela).
