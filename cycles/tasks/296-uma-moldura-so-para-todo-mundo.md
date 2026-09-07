---
id: "296"
titulo: "Uma moldura só para todo mundo, e os botões em fileira"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["295"]
estima_min: 120
---

# 296 — Uma moldura só

## O que estava errado

Duas caixas com vocabulários diferentes na mesma tela: a unidade em foco
tinha retângulo completo (ciclo 294), e o embed tinha um `└────`
desenhado na mão (ciclo 293) — meia moldura.

## O que fazer

Tirar a borda feita na mão e fazer o embed usar a MESMA moldura, com
**override de cor** e **sempre visível**. Quem desenha passa a ser o
painel, por REGIÃO; o módulo `tela` só diz de quem é cada linha.

E os botões de um embed viram fileira, como barra de ações.

## Critérios de aceite

- [x] O embed usa a mesma moldura, com cor própria
- [x] A moldura do embed aparece sempre, não só no foco
- [x] O foco desenha por cima quando os dois coincidem
- [x] A parte não desenha traço próprio
- [x] Botões viram uma fileira `[ A ]  [ B ]`

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo clippy -p anotadinho-tui --all-targets
```
