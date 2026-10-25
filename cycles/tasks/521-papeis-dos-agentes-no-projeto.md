---
id: "521"
titulo: "Papéis dos agentes no projeto: quem executa e quem revisa, pelo nome, com os perfis do aider + Ollama e do Claude revisor"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["517", "520"]
estima_min: 150
---

# 521 — O júnior executa, o tech lead revisa

Pedido: a pessoa é a PM, o Claude (Opus) é o tech lead que revisa, e um
modelo local grátis é o júnior que coda as tarefas aprovadas. Hoje todo
projeto usa o MESMO agente, o das preferências: não dá pra mandar a
execução pra um e a revisão pra outro.

Este ciclo dá os papéis. O laço de revisão (o revisor rodando sozinho
depois da verificação) é o 522; a base de problemas, o 523; os números,
o 524.

## Critérios de aceite

- [x] `ConfigProjeto` ganha `executor` e `revisor` (o NOME de um agente
      das preferências; vazio = o agente padrão / sem revisor) e
      `rodadas_de_revisao` (padrão 3). Config antiga lê igual, e a nova só
      grava o que não é padrão.
- [x] `agente::escolher(nome, opções)` é a regra única dos dois lados:
      sem nome, o padrão; com nome, o agente das opções (ou o preset) com
      esse nome; nome que não existe é ERRO dizendo qual — nunca roda
      outro agente no lugar, calado.
- [x] Perfis novos: **aider + Ollama (local)** — medido com o aider 0.86.2
      e `qwen2.5-coder:7b`: edita, commita sozinho e termina com a linha
      `RESULTADO:` — e **Claude Code (revisor)**, que lê e não edita
      (`--permission-mode default` no `-p`: o que pede permissão é negado).
- [x] A tarefa reservada diz o agente (`TrabalhoDeTarefa.agente`). A TUI
      e a janela escolhem por ele; nome que não existe fecha a tarefa como
      bloqueada com o motivo, em vez de deixá-la presa em `executando`.
- [x] `projeto ver` (CLI) mostra o executor e o revisor.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
```

## Não-objetivos

- A UI mandar a lista dos agentes da pessoa (`opcoes`) pro
  `avaliar_projetos`: fica no 526, com o harness. O backend já aceita, e
  sem ela o nome é procurado nos presets.

- A tela pra escolher os papéis (TUI e janela): ciclos 525 e 526.
- Setter no CLI: o CLI é o canal do agente, e quem decide quem revisa é
  a pessoa.
