---
title: Proposta de implementação — TUI no Windows
tags:
- proposta
- portabilidade
type: proposta
date: 2026-10-06
status: aprovada
---
# Proposta de implementação — TUI no Windows

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/specs/tui-no-windows.md
{{ /fluxo }}

## Abordagem

Fazer o que o ciclo 255 fez na janela: a decisão sobre o sistema fica
numa função pura que recebe o sistema e o ambiente **por parâmetro**, e
um invólucro fino lê o ambiente real.

- **T1/T2.** Duas funções puras na TUI:
  `pasta_de_configuracao(env, windows)` e `expandir_til(caminho, env, windows)`.
  No Windows, a pasta de configuração é `APPDATA`. No Unix fica a ordem
  de hoje (`XDG_CONFIG_HOME`, depois `HOME/.config`), e é isso que
  cumpre o RF2. A pasta do usuário é `USERPROFILE` no Windows e `HOME`
  no Unix. No Windows, `~` aceita `\` e `/` depois dele.
- **T3.** Mover `src-tauri/src/grupo_de_processos.rs` para
  `anotadinho-vault::processo`, que já é o crate que fala com o sistema
  (`caminho::recortar` mora lá). A janela, a TUI e o `cli avaliar` passam
  a chamar o mesmo `preparar` e o mesmo `encerrar_arvore`. O `libc` sai
  do `src-tauri` e entra no `vault`, só pra Unix.
- **T4.** Trocar o `localtime_r` por `chrono::Local`. O `chrono` com
  `clock` já está na árvore de dependências pelo `vault`, e é ele que a
  janela usa pra carimbar o registro de execuções (ciclo 441). Fica um
  caminho só nos três sistemas, exercido no Linux, e o `libc` sai da
  TUI.
- **T5.** `caminho_nativo(vault, relativo, windows)` troca `/` por `\` no
  Windows antes de chamar o `explorer`.

## Alternativas descartadas

- **Crate `dirs`/`directories` pro T1.** Resolveria, mas é uma
  dependência pra duas variáveis de ambiente, e esconde a decisão que o
  teste precisa ver.
- **Job object no T3.** O motivo é o mesmo do ciclo 255: é FFI que
  ninguém consegue medir aqui. A troca continua local à função.
- **Manter `localtime_r` e acrescentar `GetLocalTime` só no Windows.**
  Seriam dois caminhos, e o do Windows nunca seria exercido.

## Riscos

- `chrono::Local` no Linux lê `TZ` e `/etc/localtime`, igual à libc. O
  fuso não deve mudar, mas a troca é verificada com o teste do
  calendário que já existe.
- Mover o módulo de processos mexe no cancelamento da janela. Os testes
  dele vão junto, e o harness de UI confere o botão de interromper.

## Etapas

1. Módulo `processo` no `vault`, com os testes.
2. Janela, TUI e `cli avaliar` passam a usar o módulo.
3. T1, T2, T4 e T5 na TUI, com os testes de sistema por parâmetro.
4. Validação: workspace, check cruzado do Windows, harness.
