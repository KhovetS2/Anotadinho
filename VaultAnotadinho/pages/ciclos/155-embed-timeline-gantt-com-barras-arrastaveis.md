---
title: "Ciclo 155 — Embed timeline: gantt com barras arrastáveis"
type: ciclo
ciclo: "155"
status: concluida
date: 2026-08-19
prioridade: media
depende_de: ["148", "150"]
tags:
- ciclo
---

# Ciclo 155 — Embed timeline: gantt com barras arrastáveis

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Embed timeline: gantt com barras arrastáveis

## Objetivo

Gap levantado na pesquisa comparativa Notion × AppFlowy que o usuário
deixou em `nova-funcionalidade.txt`: o Notion tem Timeline View
(barras por intervalo, escala de horas a anos, arrastar e esticar); o
AppFlowy não tem nada equivalente, e o Anotadinho também não — o
calendário mostra ocupação por dia, mas não a duração de um projeto
atravessando semanas. Este embed entrega a visão de cronograma,
reaproveitando toda a mecânica de drag/resize que o calendário inline
já resolveu nos ciclos 069-071.

## Critérios de aceite

- [x] `EmbedKind::Timeline` + `{{ type: "timeline" }}`
- [x] `TimelineEmbedData { scale: TimelineScale (Week|Month|Quarter),
      source: TimelineSource (Manual|Vault), items: Vec<TimelineItem
      { title, start, end, tags, page }> }`
- [x] Componente `embeds/inline_timeline.rs`: eixo de datas no topo
      conforme a escala, uma linha por item, barra posicionada e
      dimensionada pelo intervalo `start`..`end`
- [x] Arrastar a barra horizontalmente move o intervalo inteiro
      (preservando a duração); arrastar a alça da borda esquerda/
      direita muda só aquela ponta — mesma mecânica de
      `inline_calendar.rs`, incluindo o guard de seleção de texto
      (ciclo 068) e o guard contra abrir modal fantasma ao soltar
      (ciclo 081)
- [x] Marcador vertical do dia de hoje + botão "Hoje" que recentraliza
- [x] Navegação de período (anterior/próximo) conforme a escala
- [x] Cor da barra pela primeira tag, via `badge_class`/`BADGE_PALETTE`
- [x] `source: vault`: itens montados de `api::scan_vault` a partir de
      `start`/`end`/`due`/`date` do frontmatter, SOMENTE LEITURA —
      clique abre a página de origem, arraste desabilitado (mesma
      regra do `CalendarSource::Vault` no calendário inline)
- [x] Item sem `end` vira barra de 1 dia; item sem `start` não é
      plotado e aparece numa gaveta "sem data" (mesmo padrão do ciclo
      072). Agendar pela gaveta é por CLIQUE (põe no início do período
      visível), não por arraste: arrastar de fora pra dentro da trilha
      exigiria um segundo sistema de arraste, e o clique resolve o
      mesmo caso em um gesto
- [x] `data-nav-item`/`data-nav-group` em toda a barra de controles,
      nas barras e nos itens da gaveta; foco visível na barra
- [x] Testes: round-trip de todas as escalas/fontes; cálculo de
      posição/largura da barra (função pura `bar_span(start, end,
      window_start, window_end) -> (offset_pct, width_pct)`), com
      intervalo parcialmente fora da janela recortado nas bordas

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Dependências entre tarefas (setas ligando o fim de uma ao início de
  outra) e reagendamento em cascata — é a feature mais cara da
  Timeline do Notion; entra depois se pedirem
- Sub-itens aninhados
- Tabela sincronizada ao lado da timeline com agregados
- Escalas de hora e de ano (semana/mês/trimestre cobrem planejamento
  de spec, que é o caso de uso do vault)

## Notas

`cargo test -p anotadinho-core`: 139 (132 + 7 novos). `cargo test
--workspace`, `cd ui && cargo test --lib` (26), `trunk build`, `cargo
build --manifest-path src-tauri/Cargo.toml`: OK.

**Bug encontrado e corrigido dentro do ciclo:** o primeiro arraste
funcionava visualmente mas não gravava nada. O `mouseup` lia o
deslocamento de um handle de `use_state` capturado quando o efeito foi
criado — congelado em 0, o mesmo modo de falha já documentado no
`edited_ref`/`pending_flush_ref` do editor. Agora o valor vive num
`use_mut_ref` (que o `mouseup` lê) espelhado num `use_state` (que
redesenha a pré-visualização).

Validação ao vivo (MCP `tauri`): inserido por `/cronograma`; eixo com
marcas semanais e linha de hoje; arrastar a barra 7 dias moveu de
19/08 pra 26/08 preservando a duração; arrastar a borda direita +3
dias esticou até 04/09; salvo e conferido no disco. Modo Vault listou
as 6 páginas do vault com `date::`, sem "+ etapa" e sem alças de
arraste (somente leitura).

`bar_span` é função pura e testável de propósito — o resto é DOM. Datas
continuam `YYYY-MM-DD` string, com a aritmética em `ui/src/date_util.rs`
(que já tem 9 testes).

## Resultado

# Ciclo 155 - done

## Resumo

`{{ type: "timeline" }}` — o gap levantado na pesquisa comparativa
Notion × AppFlowy que o usuário deixou em `nova-funcionalidade.txt`: o
calendário mostra ocupação por dia, nada mostrava duração atravessando
semanas. Barras por intervalo, escala Semana/Mês/Trimestre, arrastar
(preserva a duração), redimensionar pelas bordas, linha de hoje,
gaveta de itens sem data, e modo Vault somente leitura montado do
frontmatter.

## Arquivos criados/modificados

- `crates/core/src/embed.rs` — `TimelineScale`, `TimelineSource`,
  `TimelineItem`, `TimelineEmbedData`, `bar_span` + 7 testes
- `crates/core/src/index.rs` — braço do novo tipo
- `ui/src/components/embeds/inline_timeline.rs` (novo)
- `ui/src/components/embeds/mod.rs` — registro + dispatcher
- `ui/src/styles/main.css` — `.timeline*`

## Testes adicionados

- round-trip com tags e escala
- `bar_span`: posição/largura dentro da janela; recorte nas duas
  bordas; fora da janela e sem início devolvem `None`; intervalo
  invertido vira 1 dia (nunca barra de largura negativa)
- mover preserva a duração; redimensionar nunca inverte as pontas
- item sem data ganha só o início ao ser agendado

## Problemas encontrados

- Arraste movia a barra na tela mas gravava nada: o `mouseup` lia um
  handle de `use_state` capturado na criação do efeito — congelado em
  0. Corrigido com `use_mut_ref` pro valor commitado + `use_state` só
  pra pré-visualização (mesmo padrão do `edited_ref` do editor). Achado
  na validação ao vivo; testes de unidade não pegariam isso.
- Agendar item da gaveta ficou por clique, não arraste (ver task).

## Notas para próximos ciclos

- Dependências entre tarefas (setas + reagendamento em cascata) segue
  fora de escopo — é a parte mais cara da Timeline do Notion.
- Falta da série: 156 (actions), 157/158 (CLI), 159 (toolbar), 160
  (painel).
