---
title: Prompt de qualquer tamanho chega inteiro ao agente
type: spec
date: 2026-10-06
status: rascunho
prioridade: alta
tags:
- spec
- portabilidade
- agente
---
{{ type: "fluxo" }}
artefato: spec
etapa: rascunho
{{ /fluxo }}
## Por que isto existe

Hoje o prompt vai pro agente como **um argumento** da linha de comando.
Isso tem dois limites, e o Anotadinho estoura os dois:

**No Windows não funciona nunca.** `claude`, `codex` e `opencode`
instalados pelo npm são shims `.cmd`. A biblioteca padrão do Rust, desde
a 1.77.2, **recusa** argumento com `\r` ou `\n` quando o alvo é
`.bat`/`.cmd`. Ela devolve `batch file arguments are invalid`
(`make_bat_command_line` em `library/std/src/sys/args/windows.rs`, lido
no código-fonte). Todo prompt que o Anotadinho monta tem quebra de
linha, porque a blindagem do conteúdo do vault é feita com marcadores em
linhas próprias. Então, no Windows, **nenhuma** conversa chega a
executar. Além disso, o `cmd.exe` corta a linha de comando em 8191
caracteres.

**No Linux falha com prompt grande.** Um argumento sozinho não pode
passar de 128 KiB (`MAX_ARG_STRLEN`), e o spawn falha com `E2BIG`. Uma
conversa com algumas páginas transcluídas chega a esse tamanho.

O ciclo 255 resolveu o shim `.cmd` (item B1) e escreveu, com razão, que
"compila não é o mesmo que roda". Este é o primeiro problema que
aparece ao rodar.

## Requisitos funcionais

1. **RF1** — Um prompt com quebras de linha chega inteiro ao agente no
   Windows, inclusive quando o agente é um shim `.cmd`/`.bat`.
2. **RF2** — Um prompt de qualquer tamanho que o agente aceite chega
   inteiro, nos três sistemas. O limite deixa de ser o da linha de
   comando.
3. **RF3** — Os perfis prontos de agentes que sabem receber o prompt por
   outro meio passam a usar esse meio.
4. **RF4** — Quem já tem um perfil pronto gravado recebe a mudança sem
   reconfigurar nada. Um perfil que a pessoa montou à mão continua do
   jeito que ela montou.
5. **RF5** — A pessoa vê e escolhe como o prompt é entregue, na tela de
   configuração da janela e na da TUI.
6. **RF6** — Quando a forma de entrega escolhida não funciona no sistema
   (prompt com quebra de linha indo como argumento pra um `.cmd`), a
   mensagem de erro diz isso e diz o que mudar. Não pode ser
   `batch file arguments are invalid`.
7. **RF7** — Continua sem shell no caminho. O conteúdo do prompt nunca é
   interpretado como comando, qualquer que seja a forma de entrega.

## Requisitos não funcionais

- Vale pra todo lugar que dispara agente: a janela, a TUI e o
  `anotadinho-cli avaliar`.
- O registro de execuções continua medindo o tamanho do prompt enviado.

## Ambiguidades (precisam de decisão)

- **opencode.** O `opencode run --help` não diz se ele lê o prompt do
  stdin. Sem medir num binário com modelo configurado, o perfil dele
  fica como está e o RF1 **não** vale pra ele. Isso é aceitável, ou o
  opencode bloqueia a spec?

## Critérios de aceite

- [ ] Medido em 2026-10-06 no Linux: `claude -p` e `codex exec` sem
      prompt posicional leem o prompt de várias linhas do stdin e
      respondem.
- [ ] Os perfis prontos do Claude Code e do Codex entregam o prompt pelo
      stdin; os argumentos não contêm o prompt (teste unitário).
- [ ] Um perfil gravado igual ao perfil pronto atual é migrado; um perfil
      com argumentos próprios não é (teste unitário).
- [ ] Um prompt de 300 KiB chega inteiro a um agente de teste que ecoa o
      stdin (teste com processo de verdade, no Linux).
- [ ] Argumento com quebra de linha pra um `.cmd` dá uma mensagem que cita
      a opção de entregar pelo stdin (teste unitário da tradução do
      erro).
- [ ] As telas de configuração da janela e da TUI mostram e editam a
      forma de entrega.
- [ ] Harness de UI verde.
- [ ] **Manual, numa máquina Windows (pendente):** uma conversa com
      `claude` instalado pelo npm responde.

## Fora de escopo

- Trocar o formato de entrada pra `stream-json` (`--input-format`).
- Conversa interativa. Continua um disparo por pergunta.
