---
title: Montar a base do projeto
type: prompt
date: 2026-10-22
tags:
- prompt
- projeto
---

Monte a base deste projeto: {{assunto}}

1. Chame `ver_projeto()` (ou `anotadinho-cli projeto modelo-de-plano`)
   e leia o MODELO de plano. É o único formato aceito, e um campo com
   nome errado é recusado.
2. Escreva o plano: a visão, o objetivo, a stack e os combinados do
   projeto, a arquitetura, de 2 a 4 sprints com objetivo claro, e as
   tarefas de cada sprint. Cada tarefa tem objetivo, critérios de
   aceite VERIFICÁVEIS, abordagem e fora de escopo, e cabe numa
   execução só do agente.
3. Proponha com `propor_plano_de_projeto` (ou `projeto montar --file`).
   Nada é gravado: eu reviso o lote e aplico.

Não invente requisito que eu não pedi. Se algo estiver vago demais pra
virar tarefa, diga o que falta em vez de decidir por mim.
