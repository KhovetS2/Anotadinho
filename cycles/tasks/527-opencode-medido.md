---
id: "527"
titulo: "opencode medido: o dialeto JSON dele no leitor de stream, o prompt pelo stdin e o consumo no registro"
status: done
criado: 2026-10-25
autor: agente
prioridade: media
depende_de: ["521", "524"]
estima_min: 90
---

# 527 — O opencode como júnior (ou tech lead)

O perfil do opencode era `run {prompt}` em texto, e nunca tinha sido
conferido contra o programa rodando de verdade (o comentário dele dizia
isso). Medido em 2026-10-25 com o opencode 1.18.14:

- `--format json` emite eventos: `text` (`part.text`), `tool_use`
  (`part.tool` e `part.state.input`), `step_finish` (`part.tokens.input`,
  `.output`, `.reasoning`, `.cache.read/write` e `part.cost`) e `error`
  (`error.data.message`), todos com `sessionID`;
- sem mensagem no argumento, ele lê o prompt do STDIN;
- `--auto` aprova as ferramentas, o que uma execução sem ninguém no
  terminal precisa;
- o modelo padrão desta máquina (plano opencode-go) respondeu 401 (chave
  expirada); o `opencode/big-pickle` (grátis) editou, testou, commitou e
  fechou com `RESULTADO: concluida` em 14 s.

## Critérios de aceite

- [x] `LeitorStream` entende o dialeto do opencode: a resposta é o ÚLTIMO
      `text`, o progresso mostra as ferramentas, o uso SOMA os
      `step_finish` (cada passo conta o seu) e o erro vem de
      `error.data.message`. Teste com as linhas medidas.
- [x] O perfil `opencode` passa a ser `run --format json --auto -m
      opencode/big-pickle`, pelo stdin, em `StreamJson`. O perfil antigo
      gravado (`run {prompt}`) migra sozinho, como os outros.
- [x] Os números por papel (524) passam a ter tokens e custo do opencode.
- [x] Pela TUI de verdade: o opencode como executor de uma tarefa do
      projeto (piloto), até a Revisão.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Continuar a sessão do opencode (`-s`): o app põe o par da sessão ANTES
  dos argumentos, e no opencode ele precisa vir depois do `run`. Sem
  sessão, o histórico vai no prompt, como no aider.
