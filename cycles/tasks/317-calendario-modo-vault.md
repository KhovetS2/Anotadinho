---
id: "317"
titulo: "O calendário em modo vault na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["316"]
estima_min: 90
---

# 317 — O calendário em modo vault

## O que motivou

Última peça do calendário da janela que faltava no terminal: `mode:
vault`, em que os eventos são as páginas do vault com `date::` (e
`end_date`, `time`), e clicar num evento abre a página dele. Na TUI um
calendário assim aparecia vazio.

## O que mudou

- **A tradução de página em evento foi pro núcleo**
  (`calendario::entradas_do_vault`). Morava na janela, colada à chamada
  de IPC; a janela agora chama a mesma função. A varredura continua sendo
  de quem roda.
- **Evento do vault aponta pra página**: parte "pagina" escondida, no
  evento da grade e no compromisso da agenda; o detalhe já mostrava o
  caminho (314).
- **TUI**: o `main` varre o vault uma vez (`handle_scan_vault`) e entrega
  os eventos ao `Estado` (`com_eventos_do_vault`). Calendário em modo
  vault é montado com eles, nas mesmas visões e teclas dos outros. Enter
  num evento do vault abre a página e seleciona ela na barra lateral.

## O que não entrou

A varredura é feita ao abrir a TUI; página com data criada ou mudada
depois não aparece até reabrir. A janela revarre a cada exibição.

## Critérios de aceite

- [x] Calendário `mode: vault` mostra as páginas com data
- [x] Detalhe do evento mostra a página; Enter abre ela
- [x] Evento escrito no embed não abre nada
- [x] A janela usa a mesma tradução de página em evento

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored
cargo clippy -p anotadinho-tui -p anotadinho-core --all-targets
cargo check --target wasm32-unknown-unknown --manifest-path ui/Cargo.toml
```
