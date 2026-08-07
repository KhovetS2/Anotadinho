---
id: "066"
titulo: "Calendario embedado grade mensal e date picker proprio"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: ["065"]
estima_min: 120
agente_alvo: claude-sonnet
---

# Calendário embedado como grade mensal + date picker próprio

## Objetivo

Dois problemas resolvidos juntos: o `<input type="date">` nativo tinha um
bug real no WebKitGTK (popup não fechava sozinho — contornado com
`.blur()` no ciclo anterior, um remendo) e destoava visualmente do resto
da UI. E o calendário embedado (`{{ type: "calendar" }}`) nunca foi de
fato uma grade — era só uma lista agrupada por data. Este ciclo entrega
um `DatePicker` com identidade visual própria (substitui o nativo em
todo canto) e reformula o calendário embedado como grade mensal de
verdade, inspirada nas visões de calendário do Notion/AppFlowy: eventos
com intervalo de datas (barras contínuas na grade), cores por tag,
arrastar-e-soltar pra reagendar.

## Critérios de aceite

- [x] `ui/src/date_util.rs` (novo): matemática de data pura via número de
      dia juliano (sem dependência nova — `chrono` foi removida do
      projeto deliberadamente)
- [x] `ui/src/components/date_picker.rs` (novo): popover de calendário
      reutilizável — navegação de mês, dia de hoje destacado, atalho
      "Hoje", fecha em outside-click/Escape
- [x] Célula Data da tabela usa o `DatePicker` em vez do
      `<input type="date">` nativo — resolve o bug de fechamento pela
      raiz (não depende mais de nenhum widget do SO)
- [x] `CalendarEntry` ganha `end_date`/`tag` opcionais (retrocompatível —
      entradas antigas sem esses campos continuam parseando), mais
      `update_entry`/`move_entry` (desloca `date` preservando duração)
- [x] `EventDetailModal` (novo): título, início/fim via `DatePicker`,
      toggle "Vários dias", tag (chips reaproveitados + criar nova),
      excluir
- [x] `inline_calendar.rs` reescrito como grade mensal: eventos de 1 dia
      e com intervalo unificados como barras (algoritmo guloso de lanes
      por semana, máx. 3 visíveis + "+N mais"), clicar em dia vazio cria
      evento rápido, clicar numa barra abre o modal, arrastar reagenda
      preservando duração (mesmo padrão de mouse do kanban, com listener
      global de `mouseup` pra nunca travar o drag)
- [x] `badge_class`/`BADGE_PALETTE` viram compartilhados (`ui/src/embed.rs`)
      entre tabela e calendário, em vez de duplicados

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Visões semana/dia, Gantt/Timeline com dependências — só grade mensal
- Sincronização externa (Google/Outlook/Apple Calendar), links de
  agendamento, notificações — fora do escopo de um app local-first
- Redimensionar evento arrastando a borda — só mover o evento inteiro
- Painel de "eventos sem data" — `date` continua obrigatório
- Página de calendário inteira (`type: calendar` no frontmatter,
  `ui/src/components/calendar.rs`) continua como lista — só o embed
  inline ganhou a grade

## Notas

Validação ao vivo via MCP `tauri` pegou um alerta falso: a grade abriu
mostrando "Setembro 2026" como mês inicial em vez de "Agosto 2026" (mês
real). Investigação mostrou que não era bug — era estado de um mount
anterior do componente que sobreviveu a um hot-reload do trunk durante a
sessão de testes. Fechar e reabrir a aba (forçando remount) mostrou
"Agosto 2026" corretamente. Lição: ao testar componentes com estado
inicializado a partir de `js_sys::Date` (data "de hoje"), fechar/reabrir
a aba antes de validar evita falso-positivo de hot-reload.

Testado e confirmado ao vivo: evento de intervalo (`Sprint de agosto`,
10-14 ago) renderiza como barra contínua de 5 colunas; arrastar esse
mesmo evento de intervalo pra outro dia preserva a duração exata (moveu
pra 25-29 ago); arrastar um evento de 1 dia move só a data; clicar
(sem arrastar) abre o modal; "Vários dias" revela o campo Fim; o
`DatePicker` da tabela abre, mostra o mês certo com hoje destacado, e
FECHA sozinho ao escolher uma data — resolve de vez o bug do ciclo
anterior, já que não existe mais popup nativo nenhum envolvido.
