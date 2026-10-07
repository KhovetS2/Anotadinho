---
id: "453"
titulo: "Revisão na TUI: o diff quebra a linha, e a Visualização mostra o que será gravado"
status: done
criado: 2026-10-07
autor: agente
prioridade: alta
depende_de: ["452"]
estima_min: 90
---

# 453 — O diff da TUI mostra a mudança

Spec: `pages/specs/revisao-de-proposta.md` (RF2, RF3, RF4).

## Objetivo

O diff da TUI cortava cada linha na largura do cartão. Numa linha longa
de markdown, as versões `-` e `+` saíam idênticas, porque a mudança
ficava depois do corte. A Visualização mostrava a proposta inteira,
mesmo com trechos tirados. O cabeçalho ("Visualizaç") e os atalhos
também eram cortados.

## Critérios de aceite

- [x] A linha do diff quebra: a continuação tem recuo, o fundo segue
      até a borda e a palavra mudada continua realçada (teste lendo o
      estilo do buffer)
- [x] A Visualização mostra o conteúdo parcial (teste)
- [x] Os modos e os atalhos descem de linha quando não cabem
- [x] Os testes novos reprovam com o `especiais.rs` antigo
- [x] TUI de verdade num pty de 100×40: diff, Visualização e
      Visualização sem o trecho 1

## Comandos de validação

```bash
cargo test -p anotadinho-tui
```

## Não-objetivos

- O foco na barra lateral depois de abrir as propostas pela barra de
  comandos (já era assim; `Tab` resolve)
