---
id: "312"
titulo: "Eventos navegáveis entre meses no calendário"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["311"]
estima_min: 60
---

# 312 — Eventos navegáveis entre meses

## O que motivou

O 311 fez `h`/`l` andarem de dia com o evento selecionado, mas parava
na ponta do mês: do último evento de agosto não se chegava no primeiro
de setembro.

## O que mudou

`tela::evento_ao_lado` passou a contar por DATA, e não por posição na
grade: junta os dias de todos os meses do calendário, em ordem, e anda
pro próximo (ou anterior) que tem evento.

Só entram os dias DO PRÓPRIO mês de cada grade. As células de fora (o
1º de setembro no fim da grade de agosto) são os mesmos dias que a
grade seguinte mostra; contadas duas vezes, virar o mês faria o cursor
voltar no tempo — de 5 de setembro (fim da grade de agosto) pra 30 de
agosto (começo da de setembro).

A data de cada dia sai de `calendario::month_cells` com o ano e o mês
lidos do rótulo do mês (`tela::ano_e_mes`), que usa a mesma
`date_util::month_name` que o núcleo usou pra escrever o rótulo.

Dentro do mesmo mês, a mesma faixa continua preferida (a barra segue
selecionada); ao trocar de mês, o cursor vai pro primeiro evento do
dia — as faixas são refeitas por semana, e a mesma posição na grade
seguinte pode ser outro evento.

## Critérios de aceite

- [x] `l` no último evento do mês vai pro primeiro dia com evento do
      próximo mês com grade
- [x] `h` faz o caminho inverso
- [x] Evento que atravessa a virada é percorrido dia a dia, sem pular
      nem voltar no tempo
- [x] Mês sem evento (sem grade) é pulado

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
```
