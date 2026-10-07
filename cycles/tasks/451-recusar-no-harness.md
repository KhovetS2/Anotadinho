---
id: "451"
titulo: "Harness: recusar proposta passa pelo modal do motivo"
status: done
criado: 2026-10-07
autor: agente
prioridade: media
depende_de: ["449"]
estima_min: 30
---

# 451 — Recusar no harness

## Objetivo

Dois cenários, "proposta: recusar descarta sem escrever nada (204)" e
"aviso: proposta pendente aparece no cabeçalho e some ao resolver
(210)", reprovavam desde o ciclo 428. Naquele ciclo, recusar passou a
pedir o motivo: o botão do item virou "Recusar…" e só abre o modal. Os
cenários procuravam "Recusar" e não achavam.

## Critérios de aceite

- [x] `recusarDeTeste` clica "Recusar…", espera o botão do modal (o
      elemento que vai tocar) e confirma
- [x] Os dois cenários passam
- [x] Os outros cenários de proposta e revisão continuam verdes

## Comandos de validação

```bash
node scripts/uitest/run.mjs "recusar descarta"
node scripts/uitest/run.mjs "proposta pendente aparece"
node scripts/uitest/run.mjs "proposta:"
node scripts/uitest/run.mjs "revisão"
```

## Não-objetivos

- Os outros três cenários que reprovam ("controles da janela",
  "aparência: voltar ao padrão", "prompt padrão: preview")
