---
id: "311"
titulo: "h/l andam entre os dias com o evento selecionado"
status: done
criado: 2026-09-16
autor: agente
prioridade: media
depende_de: ["310"]
estima_min: 60
---

# 311 — h/l andam de dia com o evento selecionado

## O que motivou

Com o cursor num evento (309, 310), `h`/`l` não faziam nada: os irmãos
de um evento são as outras faixas do MESMO dia, empilhadas, e num galho
em coluna `h`/`l` não têm pra onde ir. Pra ir ao dia ao lado era
preciso Backspace, `l`, Enter.

## O que mudou

`tela::evento_ao_lado` responde "qual evento no dia vizinho". Com o
cursor num evento do calendário, `h`/`l` andam de dia e o cursor
continua num evento — o nível não muda, a regra do ciclo 302.

- No dia ao lado a preferência é a MESMA faixa. Dentro de uma barra de
  vários dias ela é a continuação, então `l` percorre a sprint dia a
  dia com ela selecionada, sem pular pra outro evento que divide o dia.
- Se a faixa ali não é evento, o cursor vai pro primeiro evento do dia.
- Dia sem evento é pulado; a semana vira junto (sábado → domingo),
  dentro do mesmo mês.
- Sem evento adiante, o cursor fica. Contagem vale (`3l`).

## Critérios de aceite

- [x] `l`/`h` num evento vão pro evento do dia vizinho
- [x] Dentro da barra, a mesma faixa (a própria barra) é preferida
- [x] Faixa vazia no vizinho cai no primeiro evento do dia
- [x] Dias sem evento são pulados, atravessando a semana
- [x] Na ponta o cursor fica; contagem funciona

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
```
