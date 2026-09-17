---
id: "321"
titulo: "Editar a tabela pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["318"]
estima_min: 60
---

# 321 — Editar a tabela pela TUI

## O que mudou

Sobre a base do 318. A célula já tem endereço na árvore (fileira 0 é o
cabeçalho, as outras são as linhas de dados), então não precisou de parte
escondida nova.

- `c` — edita a célula, com o valor atual na pergunta. No cabeçalho,
  renomeia a coluna. Num select, um valor que ainda não é opção vira
  opção; num multiselect, as tags vão separadas por vírgula e as novas
  também viram opção — a criação de tag direto na célula da janela.
- `x` — limpa a célula.
- `o` — linha vazia embaixo da do cursor; o cursor vai pra primeira
  célula dela.
- `dd` — apaga a linha (o cabeçalho não se apaga).

Também no 320: teste de que o cronograma com `source: vault` recusa edição.

## O que não entrou

Criar/apagar/reordenar colunas, trocar o tipo da coluna e editar a lista
de opções — o modal de configuração de coluna da janela.

## Critérios de aceite

- [x] Editar célula de texto, select e multiselect, com opções novas
- [x] Renomear coluna, limpar célula, criar e apagar linha
- [x] O resto do arquivo não muda

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
```
