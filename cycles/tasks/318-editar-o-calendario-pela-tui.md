---
id: "318"
titulo: "Editar pela TUI: a base da edição de embed e o calendário"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["317"]
estima_min: 240
---

# 318 — Editar o calendário pela TUI

## O que motivou

Com os designs da janela transferidos, o próximo passo pedido é a TUI
EDITAR. O calendário, recém-completo, é o primeiro.

## A base (vale pra todo embed)

- **Texto da página no Estado.** O `main` lê o arquivo com versão
  (`handle_read_page_versioned`) e abre com `abrir_texto`. Sem texto, a
  TUI continua só leitura.
- **`editar_embed(estado, embed, mudar)`**, pura: lê os dados da FONTE
  do embed, aplica a mudança, serializa com `to_fence_text` (o que a
  janela grava) e costura no texto do arquivo pelo intervalo de bytes
  que `analisar` guarda. O resto do arquivo volta byte a byte. A árvore é
  refeita mantendo cursor, âncoras, visões e dobras.
- **Gravação pendente.** A edição deixa o texto novo em `gravacao`; o
  `main` grava com `handle_write_page_checked` (a trava de versão da
  janela). Recusada — o arquivo mudou por fora —, a página volta a ser a
  do disco e o rodapé diz por quê.
- **Pergunta no rodapé** (`Pergunta`): o texto que uma edição precisa,
  com Enter confirmando e Escape cancelando; enquanto aberta, recebe toda
  tecla, como a busca.
- **Aviso no rodapé**: vale até a próxima tecla.

## O calendário

Cada evento escrito no embed ganha a parte escondida "indice" (a
entrada no arquivo; na semana, a ordem pela hora é mapeada de volta).

- `o` — novo evento no dia do cursor (num dia, num evento, na agenda).
- `c` — renomeia o evento (o título atual vem preenchido).
- `x` ou `dd` — apaga o evento.
- `<` / `>` — move o evento um dia, preservando a duração (o arrastar da
  janela); o cursor vai junto, e a âncora também quando ele sai da grade.

Calendário em modo vault recusa, como a janela: "só leitura".

## O que não entrou

Horário, data de fim e tags ainda não se editam pela TUI; nem desfazer.

## Critérios de aceite

- [x] Criar, renomear, apagar e mover eventos, gravando o arquivo
- [x] O resto do arquivo não muda
- [x] Cursor segue o evento editado
- [x] Escape cancela sem gravar
- [x] Modo vault recusa com aviso
- [x] Gravação com trava de versão

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
```
