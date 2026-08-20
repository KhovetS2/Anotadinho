---
id: "167"
titulo: "Operar kanban e cronograma pelo teclado"
status: pending
criado: 2026-08-20
autor: humano
prioridade: media
depende_de: ["165"]
estima_min: 90
agente_alvo: claude-opus-5
---

# Operar kanban e cronograma pelo teclado

## Objetivo

Navegar até o controle já será possível depois da task 165, mas duas
operações continuam exclusivas do mouse porque são ARRASTE: mover um
card entre colunas do kanban e mover/redimensionar uma barra do
cronograma. Enquanto isso existir, "usar o app só pelo teclado" é
meia-verdade.

## Critérios de aceite

- [ ] Card de kanban focado: setas ←/→ movem de coluna, ↑/↓ reordenam
      dentro da coluna (as duas coisas já existem em
      `KanbanEmbedData::move_card`, que resolve troca de coluna e
      reordenação numa chamada)
- [ ] Barra de cronograma focada: ←/→ deslocam por dia preservando a
      duração (`TimelineEmbedData::move_item`); com Shift, redimensiona
      a borda final (`resize_item`)
- [ ] Item da gaveta "sem data" do cronograma: Enter agenda no início
      do período visível (já é o comportamento do clique)
- [ ] Cada tecla mostra o resultado na hora e grava pelo mesmo caminho
      do arraste (`on_change`), então undo (095) desfaz igual
- [ ] O que a tecla faz aparece no cheatsheet, na seção do embed
- [ ] Validação ao vivo: montar um board e um cronograma e reorganizar
      os dois inteiros sem mouse

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar com o teclado no calendário (grade de horas) — entra depois
  se o padrão daqui funcionar
- Atalho pra criar card/etapa (já tem botão, alcançável por 165)

## Notas

A lógica de mutação toda já existe e é testada no core — este ciclo é
ligar tecla nela, não reimplementar movimento.
