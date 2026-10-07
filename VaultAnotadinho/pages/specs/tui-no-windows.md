---
title: TUI no Windows
type: spec
date: 2026-10-06
status: rascunho
prioridade: alta
tags:
- spec
- portabilidade
- tui
---
{{ type: "fluxo" }}
artefato: spec
etapa: rascunho
{{ /fluxo }}
## Por que isto existe

A spec [[rodar-no-windows]] foi fechada no ciclo 255, quando só a janela
existia. A TUI (`crates/tui`) nasceu depois, na série 342+, e não herdou
nenhuma das correções daquele ciclo. Ela compila para
`x86_64-pc-windows-gnu`, mas compilar não quer dizer funcionar: quatro
comportamentos dependem de convenções do Unix e falham no Windows sem
mostrar erro.

## O que falha hoje, e como aparece pra pessoa

**T1. As preferências não são gravadas.** A pasta de configuração vem
de `XDG_CONFIG_HOME` ou de `HOME`. No Windows, `HOME` normalmente não
existe. Tudo que a pessoa configura na TUI (agente, pasta de trabalho,
pastas extras, tema) some ao fechar, e a única pista é o aviso
"não gravou as preferências: sem pasta de configuração".

**T2. `~` não é expandido.** Digitar `~\projetos` como pasta do agente ou
como vault é recusado com "não é uma pasta", pelo mesmo motivo do T1.

**T3. Interromper o agente deixa os subprocessos rodando.** A TUI encerra
só o processo filho direto. `claude` e `codex` são wrappers que criam
outros processos, e no Windows esses netos continuam rodando depois do
cancelamento e do tempo limite. A janela já resolveu isso no ciclo 255
(item D1); a TUI não.

**T4. A data é a de UTC.** "Hoje" no calendário, o carimbo das
conversas e as datas que a TUI grava usam o fuso da pessoa só no Unix.
No Windows usam UTC: em Brasília, depois das 21h, a TUI já está no dia
seguinte.

**T5. Abrir um anexo pode cair na pasta errada.** O caminho entregue ao
Explorer mistura `\` e `/` (`C:\vault\assets/x.png`). Não foi medido,
mas o Explorer é conhecido por tratar mal a barra normal.

## Requisitos funcionais

1. **RF1** — No Windows, as preferências da TUI são gravadas e lidas da
   pasta de configuração do usuário (a que o sistema indica pra dados de
   aplicativo) e sobrevivem a fechar e abrir a TUI.
2. **RF2** — Quem já tem preferências gravadas no Linux e no macOS
   continua lendo do mesmo lugar de hoje. Nada se perde na atualização.
3. **RF3** — `~` no começo de um caminho digitado na TUI vira a pasta do
   usuário, nos três sistemas.
4. **RF4** — Interromper o agente, ou o tempo limite estourar, encerra a
   árvore inteira de processos, na TUI e na janela, com o mesmo
   comportamento.
5. **RF5** — A data e a hora que a TUI mostra e grava são as do fuso
   configurado no sistema, nos três sistemas.
6. **RF6** — Abrir um anexo do vault entrega ao programa do sistema um
   caminho no formato nativo dele.

## Requisitos não funcionais

- O comportamento de cada sistema é decidido por **parâmetro**, e não
  por `cfg!` no meio da lógica, pra o teste do Windows rodar numa
  máquina Linux. O modelo é `agente::resolver_executavel`.
- A TUI e a janela não podem ter duas implementações de "encerrar a
  árvore de processos".

## Critérios de aceite

- [ ] Com `APPDATA` definido e sem `HOME`, a pasta de preferências fica
      dentro de `APPDATA` (teste unitário com o sistema por parâmetro).
- [ ] Com `XDG_CONFIG_HOME` ou `HOME` no Unix, o caminho é exatamente o
      de hoje (teste unitário).
- [ ] `~\x` no Windows vira `<USERPROFILE>\x`; `~/x` no Unix vira
      `<HOME>/x`; um caminho sem `~` não muda (teste unitário).
- [ ] A TUI encerra o agente pelo mesmo código que a janela usa.
- [ ] No Linux, cancelar um agente que criou um processo neto mata o
      neto também (teste com processo de verdade).
- [ ] O cálculo de "hoje" não depende de `libc` nem de `#[cfg(unix)]`.
- [ ] O caminho entregue ao `explorer` só usa `\` (teste unitário com o
      sistema por parâmetro).
- [ ] `cargo check --target x86_64-pc-windows-gnu` passa pra TUI, pra CLI
      e pro app.
- [ ] **Manual, numa máquina Windows (pendente):** configurar o agente na
      TUI, fechar, abrir de novo e ver a configuração; interromper uma
      execução e conferir no Gerenciador de Tarefas que não sobrou
      `node.exe`; abrir um anexo `.png` com `o`.

## Fora de escopo

- Empacotamento ou instalador da TUI para Windows.
- Área de transferência do sistema na TUI: falta nos três sistemas, não
  é um problema do Windows.
- Rodar a TUI em `cmd.exe` antigo (sem suporte a VT). O alvo é o Windows
  Terminal.
