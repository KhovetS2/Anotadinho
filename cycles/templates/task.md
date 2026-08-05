---
id: "{ID}"
titulo: "{TITULO}"
status: pending
criado: {DATA}
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# {TITULO}

## Objetivo

{Descrever em 1-3 frases o que este ciclo vai entregar. Seja específico.}

## Critérios de aceite

- [ ] {Critério 1 - testável automaticamente se possível}
- [ ] {Critério 2}
- [ ] {Critério 3}

## Comandos de validação

```bash
# Estes comandos rodam automaticamente via cycles/orchestrator.sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

## Não-objetivos

- {O que NÃO fazer neste ciclo - geralmente coisas de ciclos futuros}
- {Foco é fazer UMA coisa bem feita}

## Notas

{Qualquer contexto extra, decisões tomadas, links pra docs, etc}
