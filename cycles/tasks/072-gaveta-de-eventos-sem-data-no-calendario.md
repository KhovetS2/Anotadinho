---
id: "072"
titulo: "Gaveta de eventos sem data no calendario"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: ["071"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Gaveta de eventos sem data no calendário

## Objetivo

Quinto item do backlog dos ciclos 066/067: `CalendarEntry.date` vira
`Option<String>`, e uma gaveta recolhível no rodapé do embed do
calendário lista os eventos sem data. Dá pra criar um evento sem data
direto, arrastar da gaveta pra um dia (mesmo mecanismo de `dragging` já
usado pelas barras/blocos da grade) ou abrir o modal e definir a data
pelo `DatePicker`.

## Critérios de aceite

- [x] `CalendarEntry.date: Option<String>` — `None` = sem data, não
      serializa a chave `date:` no YAML quando `None`
- [x] `CalendarEmbedData::add_unscheduled_entry(title)`
- [x] `move_entry`/`move_entry_time` funcionam pra atribuir data a um
      evento sem data (não só pra mover um já agendado) — sem duração
      antiga pra preservar nesse caso
- [x] `pack_days` e o filtro de blocos com horário ignoram eventos sem
      data (não aparecem na grade)
- [x] Gaveta no rodapé: botão de recolher/expandir com contagem, "+
      evento sem data", lista de pills arrastáveis
- [x] Arrastar um item da gaveta pra um dia (mês) ou coluna (semana/dia)
      atribui a data, some da gaveta, aparece na grade
- [x] `EventDetailModal`: evento sem data mostra só o campo "Início"
      (funciona como "definir data"); "Vários dias"/"Horário específico"
      só aparecem depois que a data existe
- [x] Retrocompatível: entradas antigas (sempre tinham `date:`) continuam
      parseando igual, sem mudança de comportamento

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Visões Semana/Dia na página de calendário inteira (`type: calendar`)
  — fica pra retomar depois (task #59 no tracker)

## Notas

Escopo do `Option<String>` ficou menor do que parecia — o componente de
calendário de página inteira (`ui/src/components/calendar.rs`, usado no
frontmatter `type: calendar`) usa um modelo de dados totalmente separado
(`DayItem`, raspado de `date::` nas páginas), não `CalendarEntry`, então
não precisou de nenhuma mudança.

Validado ao vivo via MCP `tauri`: criar evento sem data via botão da
gaveta (Prompt), abrir a gaveta e ver a contagem/pill, arrastar o pill
pra um dia do mês — some da gaveta, aparece na grade naquele dia,
contagem de "N eventos" no cabeçalho soma certo. Testado também o outro
caminho: criar sem data, clicar (não arrastar) pra abrir o modal, ver
"Sem data — clique para definir" no lugar do chip de data, clicar nele
pra abrir o `DatePicker` e escolher uma data — campo atualiza pra
`2026-08-15`, os campos "Vários dias"/"Horário específico" aparecem, e o
evento passa a aparecer na grade no dia escolhido.

`exemplos-embeds.md` ganhou um evento sem data (`Ligar pro fornecedor`,
sem chave `date:`) pra documentar a sintaxe e servir de fixture de
regressão (`exemplos_embeds_vault_file_parses` atualizado pra 5 entradas).
