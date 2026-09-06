---
id: "279"
titulo: "Passo 4c: a borda do documento vem do modelo"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["278"]
estima_min: 90
---

# 279 — A borda do documento vem do modelo

## O defeito, medido antes de escrever código

Sonda no app de verdade: página com quatro blocos (parágrafo, lista,
embed, parágrafo), entra no editor, `j` do topo. A trilha:

```
0: p/texto/"primeiro"
1: ul/grupo/"um dois"
2: div/embed/"+dentro do callout"
3: p/texto/"ultimo"
4: p/texto/"primeiro"   ← deu a volta
```

E `k` no primeiro faz o mesmo pro outro lado.

A volta é `(i + 1) % items.len()` em `app.rs`, valendo pra TODO grupo de
navegação. O núcleo diz o contrário desde o ciclo 273: `mover` devolve
`None` na borda, e `None` significa "fica onde está".

## Por que isso importa fora do modelo

É o que a pessoa relatou ao pedir a árvore n-ária: numa CLI, sem mouse
pra corrigir, um `j` a mais que teleporta pro topo do documento faz
perder o lugar sem aviso nenhum.

Circular continua CERTO em menu — lista curta e fechada, dar a volta
ajuda. Documento não é menu, e a distinção fica explícita no código.

## Critérios de aceite

- [x] Dois cenários foram vistos REPROVANDO antes do conserto
- [x] A regra da borda mora no núcleo, não na GUI
- [x] Menu continua circulando
- [x] A suíte não muda de resultado (257/257)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
