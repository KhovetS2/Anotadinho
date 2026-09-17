---
id: "320"
titulo: "Editar o cronograma pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["318", "319"]
estima_min: 90
---

# 320 — Editar o cronograma pela TUI

## O que mudou

Sobre a base do 318. Cada barra (e cada item sem data) ganha a parte
escondida "indice" — o item no arquivo —, sem mudar o desenho.

- `o` — nova barra de 7 dias, começando no dia seguinte ao fim da barra
  do cursor (sem barra: hoje; sem hoje na janela: o começo do eixo).
- `c` — renomeia a barra.
- `x` ou `dd` — apaga a barra.
- `<` / `>` — anda a barra um dia pra trás/frente, mantendo a duração.
- `-` / `+` — encolhe/estica o fim um dia (o fim não passa do começo).

Cronograma com `source: vault` é só leitura: o rodapé avisa.

## O que não entrou

Editar tags, grupo e cor da barra; mudar o começo sem mover a barra.

## Critérios de aceite

- [x] Criar, renomear, apagar, mover e esticar barras
- [x] Fonte vault recusa edição
- [x] O resto do arquivo e os campos não editados não mudam

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
