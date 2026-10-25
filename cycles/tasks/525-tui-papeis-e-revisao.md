---
id: "525"
titulo: "TUI: os papéis e o laço de revisão na tela do projeto — executor, revisor e rodadas no formulário, o ponto do laço no cartão e os números por papel"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["521", "522", "523", "524", "519"]
estima_min: 180
---

# 525 — A TUI mostra quem executa, quem revisa e onde o laço está

Pedido: a pessoa é o PM, o Claude é o tech lead que revisa e um modelo
local é o júnior que coda. Os ciclos 521–524 puseram isso no núcleo, no
`ipc` e no CLI, mas na TUI nada aparecia: os papéis não se configuravam
pelo `o`, a tarefa revisando era um cartão "executando" igual aos outros,
e os números ficavam só no `projeto numeros`.

## Critérios de aceite

- [x] O formulário do `o` ganha Executor, Revisor (texto com o NOME do
      agente; a dica lista os que valem — os agentes da pessoa e os
      presets) e Rodadas (número ≥ 1; vazio = 3). Nome que o
      `agente::escolher` não acha é recusado com o formulário aberto e a
      lista dos que há; rodada zero ou que não é número também. A pausa e
      a concorrência voltam como estavam (os testes do 519 seguem).
- [x] O cabeçalho do projeto diz "executor X · revisor Y (até N
      rodadas)" (ou "· sem revisor — vai direto pra você"), e nada quando
      é tudo padrão e sem revisor. Cabe em 80 e 150 colunas (corte com `…`).
- [x] Selo no cartão e na faixa da tarefa: em `executando`, "⟳ revisão
      N/T" (pendente/rodando) e "↺ ajustes" (ajustes/refazendo); em
      `revisao`, "✓rev", "⚠teto" e "?rev". Cores do tema. A faixa usa o
      teto do projeto (`Estado::configs_dos_projetos`, lido pelo `main`).
- [x] `v` diz a revisão pelo agente (estado e rodada) e oferece "Abrir a
      conversa do revisor" quando ela existe e "Base de problemas (N)"
      quando a base existe.
- [x] `x` mostra no topo o resumo de `handle_numeros_do_projeto`: executor,
      revisor (e conversas, quando houve), com execuções, falhas, tempo,
      tokens e o custo só do que contou custo; e a revisão. Enter no
      resumo não fecha a lista. O IO é do `main`, no `Pedido`.
- [x] `docs/paridade-gui-tui.md` com a linha dos papéis.
- [x] Testes com `TestBackend` pra cada critério; dois quebrados de
      propósito reprovaram.
- [x] Conferido num pty (`scripts/tui/tela.py`) em 80 e 150 colunas.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo test --workspace
PATH=~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
python3 scripts/tui/piloto-projeto.py && python3 scripts/tui/piloto-projeto.py --falha && python3 scripts/tui/piloto-projeto.py --revisor
python3 scripts/tui/tela.py --abrir "Meu primeiro projeto" --preparar x.sh --execucoes reg.jsonl --teclas "x"
```

## Não-objetivos

- A janela: é o ciclo 526.
- Escolher o agente por uma lista (`Valor::Opcoes`): o pedido é texto
  com o nome, como a config guarda.
- Mudar o núcleo ou o `ipc`: tudo que a tela mostra já existia.
