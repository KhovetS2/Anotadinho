---
id: "188"
titulo: "Busca que enxerga dentro dos embeds"
status: pending
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [149, 150]
estima_min: 120
agente_alvo: claude-opus
---

# Busca que enxerga dentro dos embeds

## Objetivo

A busca varre markdown cru. Procurar "Tarefa 2" acha o YAML do kanban e
abre a página, mas não diz que aquilo é um CARD, não mostra em que
coluna está e não leva até ele. Este ciclo indexa o conteúdo
estruturado dos embeds e devolve resultado com tipo e destino.

## Critérios de aceite

- [ ] `EmbedData::search_entries()` em `crates/core/src/embed.rs`:
      devolve `Vec<EmbedHit { kind, label, contexto, indice }>` pra cada
      tipo (card do kanban, linha da tabela, evento do calendário, item
      da timeline, botão de ação, imagem da galeria, corpo do callout e
      das colunas).
- [ ] `crates/search` passa a indexar esses registros junto do texto.
- [ ] Resultado da busca mostra o tipo ("card em Backlog", "linha de
      tabela") em vez de uma fatia de YAML.
- [ ] Abrir um resultado de embed rola até o embed e o destaca.
- [ ] `anotadinho-cli search` mostra a mesma informação, com `--json`.
- [ ] Testes de `search_entries` pros 9 tipos.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Busca fuzzy ou ranqueamento novo — o motor de casamento continua o
  mesmo, muda o que entra nele.
- Editar direto do resultado da busca.

## Notas

O YAML cru DEVE sair do índice quando o embed é indexado de forma
estruturada, senão cada card aparece duas vezes (uma como registro,
outra como linha de YAML).
