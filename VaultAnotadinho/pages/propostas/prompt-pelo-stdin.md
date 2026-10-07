---
title: Proposta de implementação — prompt pelo stdin
tags:
- proposta
- agente
type: proposta
date: 2026-10-06
status: em-revisao
---
# Proposta de implementação — prompt pelo stdin

{{ type: "fluxo" }}
artefato: proposta
etapa: em-revisao
origem: pages/specs/prompt-de-qualquer-tamanho.md
{{ /fluxo }}

## Abordagem

Entregar o prompt pela **entrada padrão** do processo, que não tem os
limites de escape nem de tamanho da linha de comando. Continua sem
shell: os bytes vão por um pipe, e ninguém os interpreta como comando.

Medido no Linux em 2026-10-06:

- `claude -p --output-format stream-json --verbose`, **sem** prompt
  posicional e com o prompt de duas linhas no stdin, respondeu o pedido.
- `codex exec --json -`, idem. O `--help` dele diz: "If not provided as
  an argument (or if `-` is used), instructions are read from stdin".

### No núcleo

- Um campo novo no `Adaptador`, `entrada: EntradaPrompt`, com dois
  valores: `Argumento`, o padrão (comportamento de hoje, e o que lê
  configuração antiga sem o campo), e `Stdin`.
- `montar_execucao(prompt, mcp, sessao) -> Execucao { args, entrada }`:
  - em `Argumento`, igual a `montar_args_completo`, com `entrada: None`;
  - em `Stdin`, o argumento que é exatamente `{prompt}` sai da lista e
    `entrada: Some(prompt)`. O agente que precisa de um marcador pra ler
    o stdin (o `-` do Codex) recebe esse marcador nos próprios
    argumentos do perfil. Não precisa de campo novo pra isso.
- `validar`: em `Stdin`, `{prompt}` pode faltar. Se aparecer, tem que
  ser o argumento inteiro, porque `--msg={prompt}` não tem tradução pro
  stdin. Isso vira um problema novo.
- Os perfis prontos do Claude Code (`-p` sem posicional) e do Codex
  (`exec --json … -`) passam a usar `Stdin`. O opencode fica em
  `Argumento` até ser medido.
- `migrado()`: os argumentos do perfil pronto ATUAL entram na lista de
  antigos, e a migração passa a levar o campo `entrada` junto. Um perfil
  montado à mão não é tocado, e isso cumpre o RF4.
- `explicar_erro_de_spawn(erro, entrada)`: a mensagem
  `batch file arguments are invalid` vira uma frase que manda trocar a
  entrada pra stdin (RF6).

### Nos três lugares que disparam agente

`anotadinho-vault::processo::entregar_entrada(filho, texto)` escreve
numa thread própria e fecha o pipe. A thread é necessária porque um
prompt grande enche o buffer do pipe, e se a escrita bloqueasse a thread
que lê a saída, o resultado seria um deadlock. Fechar o pipe é o que
avisa o agente que o prompt acabou. Com `Argumento`, o stdin continua
`null`, como hoje.

### Nas telas

Um seletor "Entrega do prompt: argumento / entrada padrão (stdin)" na
configuração do agente da janela e da TUI, ao lado de "Formato da
saída".

## Alternativas descartadas

- **Chamar o `node` direto, sem passar pelo shim.** Teria que ler o
  `.cmd` pra achar o script. É frágil e depende de como o npm gera o
  shim.
- **Arquivo temporário com o prompt.** Nem todo agente aceita, e vira
  mais um arquivo pra limpar.
- **Escapar a quebra de linha pro `cmd.exe`.** Não existe escape seguro.
  É exatamente por isso que a std recusa.

## Riscos

- O registro do padrão "Agente e execução" diz "o prompt como UM
  argumento". A regra de segurança por trás disso (sem shell) continua
  valendo. O texto do padrão precisa ser atualizado junto, por
  proposta.
- `--resume` com stdin: o `claude` aceita, mas isso só foi medido sem
  sessão.

## Etapas

1. Núcleo: campo, `montar_execucao`, validação, perfis, migração,
   tradução do erro, com testes.
2. `vault::processo::entregar_entrada`, com teste de 300 KiB por um
   processo de verdade.
3. Janela, TUI e `cli avaliar`.
4. Telas de configuração (janela e TUI).
5. Validação: workspace, check cruzado do Windows, harness.
