---
id: "471"
titulo: "Sprints — MCP: ver_projeto, propor_plano_de_projeto, propor_sprint e propor_tarefa"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["470"]
estima_min: 120
---

# 471 — As rotas do projeto no MCP

Terceiro ciclo da proposta `pages/propostas/sprints-no-projeto.md`. O
Claude Code e o opencode, conectados por `anotadinho-cli mcp`, passam a
ver e a montar um projeto sem adivinhar o frontmatter de cada página.

## Critérios de aceite

- [x] O `CONTRATO` ganha quatro ferramentas:
  - [x] `ver_projeto(projeto?)`: só lê. Sem `projeto`, lista os
        projetos e devolve o MODELO de plano, que é como uma LLM
        conectada só por MCP (sem shell pro CLI) descobre o formato.
  - [x] `propor_plano_de_projeto(plano, motivo?)`: o YAML do plano vira
        um lote de propostas.
  - [x] `propor_sprint(projeto, nome, objetivo?, inicio?, fim?)`.
  - [x] `propor_tarefa(projeto, titulo, corpo?, sprint?, lote?)`.
- [x] A invariante do ciclo 204 continua: só quem se chama `propor*`
      escreve, e a descrição diz que passa por revisão humana. O teste
      lista as cinco que escrevem.
- [x] Cada ferramenta tem um ramo no MCP (`toda_ferramenta_publicada_tem_quem_atenda`).
      A resposta das que propõem diz que NADA foi gravado.
- [x] `como_agir` diz como montar um projeto. O prompt padrão "Montar a
      base do projeto" é semeado e descoberto pelo seletor.
- [x] Teste de integração pelo protocolo:
      `ver_projeto()` → `propor_plano_de_projeto` → aplicar →
      `ver_projeto(projeto)` → `propor_tarefa` na sprint.

## Comandos de validação

```bash
cargo test -p anotadinho-core ferramentas
cargo test -p anotadinho-cli
cargo test --workspace
```
