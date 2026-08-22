---
id: "207"
titulo: "Home do vault, specs de trabalho e sidebar recolhida"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [206]
estima_min: 150
agente_alvo: claude-opus
---

# Home, specs e sidebar recolhida

## Objetivo

Deixar o vault pronto pra ser usado como ferramenta de trabalho: uma
página inicial que mostra o estado real, as specs do que falta, o passo
a passo do modo agêntico, e a sidebar navegável com 200+ páginas.

## Critérios de aceite

- [x] Pastas nascem RECOLHIDAS, e o que a pessoa abre continua aberto
      quando a lista recarrega.
- [x] `pages/incio.md` com atalhos e 4 consultas vivas — nenhuma vazia.
- [x] `pages/propostas.md` (`type: propostas`) pra revisão.
- [x] Spec da pendência real do ciclo 175.
- [x] Spec do uso agêntico no dia a dia.
- [x] `produto/como-usar-modo-agentico.md` — passo a passo.
- [x] 2 cenários de harness.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Implementar o que as specs propõem: elas nascem em revisão, esperando
  decisão.

## Notas

**A pendência era real.** O ciclo 175 deixou "seleção atravessando
blocos" de fora de propósito, documentado como troca conhecida do modelo
de um editável por bloco. Virou spec com uma proposta que cobre o caso
real (mover três parágrafos) sem reimplementar o motor de seleção.

**Recolher tem que acontecer na CARGA, não num efeito depois.** A
primeira versão usava `use_effect_with` sobre a lista de pastas — o que
mostra tudo aberto na primeira pintura e recolhe depois, um flash com
200 páginas. Mudou pro mesmo lote de atualização em que a lista chega.

O cenário espera a lista ASSENTAR antes de conferir: páginas e pastas
vêm de chamadas separadas, então existe um instante legítimo entre uma e
outra.
