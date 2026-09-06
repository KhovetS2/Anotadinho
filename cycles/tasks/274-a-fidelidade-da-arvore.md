---
id: "274"
titulo: "A fidelidade da árvore, e por que o passo 4 não saiu"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["271", "273"]
estima_min: 240
---

# 274 — A fidelidade da árvore

## O portão que eu não tinha medido

O passo 4 é inverter a fonte da verdade: a árvore passa a mandar e o DOM
vira projeção. Antes de começar, medi uma coisa que o ciclo 271 NÃO
tinha medido.

O 271 provou que a árvore é ESTÁVEL — reanalisar o que ela escreve dá o
mesmo resultado. Isso não é a mesma coisa que ser FIEL ao original: uma
árvore que perde informação de forma consistente é estável e errada.

**Medido: 22 de 242 páginas do vault voltavam idênticas.**

Fazer a árvore mandar nesse estado reescreveria o vault inteiro.

## O que foi corrigido

Três perdas reais, cada uma achada por medir de novo depois de corrigir
a anterior — a de cima escondia a de baixo:

- **lista numerada** virava marcador (`1.` → `-`);
- **citação de várias linhas** virava várias citações, com linha em
  branco entre elas;
- **linguagem da cerca** sumia (```` ```bash ```` → ```` ``` ````).

Uma quarta — a normalização do YAML dos embeds — é PRÉ-EXISTENTE: o
editor já faz isso ao salvar desde sempre, via `embed::join`. Não é
perda que a árvore introduza.

Cada unidade também passou a guardar o markdown exato de onde veio, e a
escrita devolve esses bytes pra quem não mudou.

**Depois de tudo: 35 de 242.**

## Por que 35 e não 242

Guardar a fonte por unidade não bastou, e o motivo é onde as FRONTEIRAS
caem, não como cada unidade é serializada.

Uma continuação indentada de item de lista vira parágrafo separado, e a
linha em branco que a escrita insere entre unidades muda o arquivo — mesmo
cada unidade voltando byte a byte.

## A conclusão

**O passo 4 está bloqueado**, e não por falta de trabalho: por falta de
um round-trip sem perdas, que é projeto próprio e não etapa desta
sequência.

O desenho que resolve está descrito no status.

## Critérios de aceite

- [x] A fidelidade é MEDIDA, não suposta
- [x] As perdas que a árvore introduz são corrigidas
- [x] A perda pré-existente é separada das minhas
- [x] O bloqueio é quantificado e o caminho de saída, desenhado

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored --nocapture
node scripts/uitest/run.mjs --arvore
```
