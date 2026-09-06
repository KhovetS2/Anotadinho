---
id: "270"
titulo: "A caça à instabilidade da suíte"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["198"]
estima_min: 180
---

# 270 — A caça à instabilidade

## O sintoma

Três vezes em uma sessão, um cenário falhou na suíte inteira e passou
isolado. Cenários diferentes a cada vez, sem relação entre eles. Uma
suíte que às vezes falha sozinha ensina a ignorar vermelho.

## O método

Medir antes de mexer. Três rodadas completas como linha de base, e a
terceira foi a que entregou o caso: **cinquenta e poucas falhas**, em
todas as baterias. Não era um cenário instável — era o app degradando ao
longo do uso, e a suíte é justamente o que o mantém sob uso.

Daí veio o reprodutor rápido: rodar 3 cenários filtrados falhava 5 em 5,
em segundos, em vez de esperar 12 minutos por uma rodada completa.

## Critérios de aceite

- [x] A causa principal identificada com medida antes/depois
- [x] Uma falha deixa EVIDÊNCIA do estado, não só a mensagem
- [x] O estado modal é normalizado entre cenários, não só no início
- [x] O que sobrar fica registrado com o que já se sabe

## Comandos de validação

```bash
node scripts/uitest/run.mjs
```
