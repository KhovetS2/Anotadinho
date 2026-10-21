---
title: Quebrar em tarefas
type: prompt
date: 2026-10-21
tags:
- prompt
- projeto
---

Leia a página do projeto, a arquitetura e o índice de tarefas anexados,
e quebre isto em tarefas: {{assunto}}

Cada tarefa deve caber numa execução só do agente e ser verificável por
si. Não repita uma tarefa que já está no índice; se uma nova depende de
outra, diga qual na seção Abordagem.

Escreva cada tarefa exatamente neste formato, uma depois da outra:

```
## Tarefa: Título curto da primeira

### Objetivo
O que precisa existir quando terminar, e por quê.

### Critérios de aceite
- [ ] algo que dá pra conferir

### Abordagem
Como fazer, dentro da arquitetura.

### Fora de escopo
- o que esta tarefa não cobre

## Tarefa: Título curto da segunda

### Objetivo
…
```

Não grave nada: eu escolho quais viram tarefa. Se o pedido for vago demais
pra quebrar, aponte o que falta em vez de inventar requisito.
