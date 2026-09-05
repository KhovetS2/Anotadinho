---
id: "269"
titulo: "As células da tabela são destinos"
status: done
criado: 2026-09-05
autor: agente
prioridade: media
depende_de: ["268"]
estima_min: 120
---

# 269 — As células da tabela

## O que faltava

O ciclo 268 deu movimento a todos os embeds, e registrou uma exceção: na
tabela dava pra chegar no "+ coluna" e no "excluir linha", e não numa
CÉLULA. Elas não eram `[data-nav-item]`.

## Critérios de aceite

- [x] As 8 células de dados viram destinos navegáveis
- [x] `j`/`k`/`h`/`l` andam entre células pela grade
- [x] `i`/`a`/Enter descem pro campo da célula
- [x] Escape sobe UM nível: campo → célula → raiz → bloco
- [x] Entrar num contêiner é pelo canto de cima à esquerda, não pela
      ordem do documento

## Por que a célula e não o campo

Marcar o `<textarea>` faria a navegação pousar direto nele — e aí `j`
digitaria "j" em vez de andar. A célula é o destino; o campo é um nível
abaixo, alcançado por `i` (que é a tecla do vim pra isso).

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
