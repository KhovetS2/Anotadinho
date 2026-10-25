---
id: "509"
titulo: "TUI: buscar dentro do bloco (/ n N) e no editor de código (Ctrl+F Ctrl+G)"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["498", "506"]
estima_min: 90
---

# 509 — Busca dentro do bloco e no editor

O que o 498 e o 506 deixaram de fora: num bloco de 200 linhas, achar uma
palavra era rolar com os olhos.

## Critérios de aceite

- [x] Dentro do bloco (Enter):
  - [x] `/` abre o termo no fundo da caixa, e Enter vai ao primeiro
        achado;
  - [x] `n`/`N` vão ao próximo e ao anterior, dando a volta;
  - [x] os achados acendem, e o atual acende mais forte;
  - [x] o fundo diz "k/N";
  - [x] a vista põe o achado à vista nas duas direções;
  - [x] o primeiro Esc limpa a busca, e o segundo sai do bloco.
- [x] No editor de código:
  - [x] `Ctrl+F` abre a busca, e Enter vai ao achado;
  - [x] `Ctrl+G` vai ao próximo;
  - [x] os achados acendem;
  - [x] digitar continua editando;
  - [x] o primeiro Esc limpa a busca, e o segundo grava e fecha;
  - [x] o mesmo vale na aba Código do construtor.
- [x] A busca não diferencia maiúscula de minúscula.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
python3 scripts/tui/tela.py busca-no-bloco.md --teclas 'j Enter / a g u l h a Enter n'
```

## Não-objetivos

- Expressão regular e substituir.
