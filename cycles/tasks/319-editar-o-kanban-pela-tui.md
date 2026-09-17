---
id: "319"
titulo: "Editar o kanban pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["318"]
estima_min: 90
---

# 319 — Editar o kanban pela TUI

## O que mudou

Sobre a base do 318 (`editar_embed`, pergunta no rodapé, gravação com
trava de versão). Cada cartão ganha a parte escondida "indice" (o item
no arquivo), continuando folha — o desenho em caixa do 304 não muda.

- `o` — novo cartão no fim da coluna do cursor (na coluna ou num cartão).
- `c` — renomeia o cartão; com o cursor na coluna, renomeia a coluna, e
  os cartões dela seguem o nome novo.
- `x` ou `dd` — apaga o cartão; o cursor fica na coluna.
- `<` / `>` — leva o cartão pra coluna anterior/seguinte (o arrastar da
  janela); o cursor vai junto. Na ponta, o rodapé avisa.

Os campos que a TUI não edita (descrição, tags, prazo, checklist)
continuam no arquivo como estavam.

## O que não entrou

Criar/apagar coluna, reordenar cartões dentro da coluna, e os campos do
modal do cartão.

## Critérios de aceite

- [x] Criar, renomear, apagar e mover cartões entre colunas
- [x] Renomear coluna atualiza os cartões
- [x] O resto do arquivo e os campos não editados não mudam

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
