---
id: "188"
titulo: "Busca que enxerga dentro dos embeds"
status: done
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

- [x] `EmbedData::search_entries()` em `crates/core/src/embed.rs`:
      devolve `Vec<EmbedHit { kind, rotulo, contexto, texto, indice }>`
      pra cada tipo (card do kanban, linha da tabela, evento do
      calendário, item da timeline, botão de ação, imagem da galeria,
      corpo do callout e das colunas).
- [x] `crates/search` indexa cada registro como documento próprio, com
      colunas `origem`/`ancora` UNINDEXED.
- [x] Resultado da busca mostra a origem ("Kanban · coluna Backlog") em
      vez de uma fatia de YAML.
- [x] Abrir um resultado de embed rola até o embed e o destaca.
- [x] `anotadinho-cli search` mostra a mesma informação, com `--json`.
- [x] Testes de `search_entries` pros 9 tipos + 3 no `crates/ipc`.

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
outra como linha de YAML). Tem teste pros dois lados: o card aparece
UMA vez, e `column` (nome de campo do YAML) não casa mais com nada.

**Bug pré-existente achado na validação:** o `has_results` da sidebar
(`!all_pages.is_empty() || filter.is_empty()`) só considerava páginas
cujo TÍTULO casava. Quando nenhum título casava — exatamente o caso em
que a busca por conteúdo é a única com algo a dizer — a sidebar caía em
"Nenhum resultado" e a seção de resultados nunca era renderizada. Ou
seja, a busca por conteúdo estava invisível na sidebar desde o ciclo 094.
Corrigido junto.

Resultado da página JÁ ABERTA precisa de tratamento próprio: o editor não
re-renderiza, então ninguém consumiria o alvo. A sidebar, que conhece o
`selected_path`, chama a revelação direto nesse caso.
