---
id: "229"
titulo: "O agente propõe o fechamento da etapa"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["204", "228"]
estima_min: 120
---

# 229 — O agente propõe o fechamento da etapa

## Objetivo

Depois que o agente implementa, a página fica eternamente "em execução":
não existe caminho para ele registrar o que fez. Virar a etapa na mão,
sempre, é a lacuna. Mas deixar um modelo virá-la sozinho apagaria o
sentido de ter revisão — então ele **propõe**.

## Critérios de aceite

- [x] `fluxo::aplicar_etapa_no_texto` move embed e frontmatter juntos
- [x] Transição que a máquina não permite é recusada, dizendo o que dá
- [x] Página sem embed de fluxo é recusada com erro claro
- [x] `anotadinho-cli etapa <pagina> --para <etapa> [--resumo <arq|->]`
      enfileira uma proposta e **não** grava o arquivo
- [x] Etapa desconhecida lista as que existem
- [x] Os prompts de execução instruem o agente a fechar por esse comando,
      e a usar `bloqueada` quando não deu
- [x] O laço completo funciona: agente propõe, revisão mostra, aplicar
      muda a página

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Avanço automático de etapa sem revisão humana
- Mudar a UI de revisão (é o ciclo 230)
