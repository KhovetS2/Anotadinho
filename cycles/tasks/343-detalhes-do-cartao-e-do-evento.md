---
id: "343"
titulo: "Paridade: detalhes do cartão e do evento num formulário"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["342"]
---

# 343 — Detalhes do cartão e do evento

- Componente `Formulario` (`componentes.rs`): campos de texto, liga/desliga,
  lista e checklist, navegados com `j`/`k` item por item, `Enter`/`a`
  editam, `c` reescreve, `o` acrescenta, `dd` apaga item, `~`/espaço
  marcam, botões no fim (em vermelho, "Excluir…"), `Esc` fecha. Depois de
  acrescentar, o cursor volta pro "+" pra emendar outro.
- `Enter` num cartão do kanban abre o `CardDetailModal` da janela: título,
  descrição, tags, vencimento (AAAA-MM-DD validado), checklist,
  comentários (a hora do comentário novo vem do relógio) e anexos
  (caminho; o nome sai do arquivo), e "Excluir cartão".
- `Enter` num evento (fora do modo vault) abre o `EventDetailModal`:
  título, início, "Vários dias" com fim, "Horário específico" com das/até
  (HH:MM validado), tags e "Excluir evento". Os campos dependentes somem
  quando o liga/desliga está desligado.
- Cada mudança grava na hora, como a janela; `u` desfaz.
