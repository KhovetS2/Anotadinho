---
id: "155"
titulo: "Embed timeline: gantt com barras arrastáveis"
status: pending
criado: 2026-08-19
autor: humano
prioridade: media
depende_de: ["148", "150"]
estima_min: 150
agente_alvo: claude-sonnet
---

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

- [ ] `EmbedKind::Timeline` + `{{ type: "timeline" }}`
- [ ] `TimelineEmbedData { scale: TimelineScale (Week|Month|Quarter),
      source: TimelineSource (Manual|Vault), items: Vec<TimelineItem
      { title, start, end, tags, page }> }`
- [ ] Componente `embeds/inline_timeline.rs`: eixo de datas no topo
      conforme a escala, uma linha por item, barra posicionada e
      dimensionada pelo intervalo `start`..`end`
- [ ] Arrastar a barra horizontalmente move o intervalo inteiro
      (preservando a duração); arrastar a alça da borda esquerda/
      direita muda só aquela ponta — mesma mecânica de
      `inline_calendar.rs`, incluindo o guard de seleção de texto
      (ciclo 068) e o guard contra abrir modal fantasma ao soltar
      (ciclo 081)
- [ ] Marcador vertical do dia de hoje + botão "Hoje" que recentraliza
- [ ] Navegação de período (anterior/próximo) conforme a escala
- [ ] Cor da barra pela primeira tag, via `badge_class`/`BADGE_PALETTE`
- [ ] `source: vault`: itens montados de `api::scan_vault` a partir de
      `start`/`end`/`due`/`date` do frontmatter, SOMENTE LEITURA —
      clique abre a página de origem, arraste desabilitado (mesma
      regra do `CalendarSource::Vault` no calendário inline)
- [ ] Item sem `end` vira barra de 1 dia; item sem `start` não é
      plotado e aparece numa gaveta lateral "sem data" (mesmo padrão
      do ciclo 072), arrastável pra dentro da grade no modo manual
- [ ] `data-nav-item`/`data-nav-group`; setas movem o item focado por
      dia, Enter abre o detalhe
- [ ] Testes: round-trip de todas as escalas/fontes; cálculo de
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

`bar_span` é função pura e testável de propósito — o resto é DOM. Datas
continuam `YYYY-MM-DD` string, com a aritmética em `ui/src/date_util.rs`
(que já tem 9 testes).
