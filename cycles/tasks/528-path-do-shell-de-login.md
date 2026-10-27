---
id: "528"
titulo: "O app aberto pelo menu acha os agentes: o PATH do shell de login entra no agente, na verificação e no git"
status: done
criado: 2026-10-26
autor: agente
prioridade: alta
depende_de: ["521"]
estima_min: 90
---

# 528 — "não consegui executar \"claude\""

Relato: usando o módulo Projeto no vault `/home/elis/vault`, a tarefa 001
do projeto `plataforma-artesaos` não rodou. O registro de execuções diz:

```
não consegui executar "codex": Arquivo ou diretório inexistente (os error 2)
não consegui executar "claude": Arquivo ou diretório inexistente (os error 2)
```

Os dois estão em `~/.local/bin`. A causa: o app instalado
(`/usr/bin/anotadinho`, aberto pelo `Anotadinho.desktop`) herda o `PATH`
da SESSÃO gráfica, que aqui é `/usr/local/bin:/usr/bin`
(`systemctl --user show-environment`). O `~/.local/bin`, o `~/.opencode/bin`,
o pyenv e o sdkman só entram no `PATH` pelo `.zshrc`/`.zprofile`, que o
menu não lê. No terminal, tudo acha; pelo menu, nada.

E não é só o binário: a verificação do projeto (`npm test`, `cargo test`)
e as ferramentas que o agente chama rodam com o mesmo `PATH` pobre.

## Critérios de aceite

- [x] Fora do Windows, o app pergunta uma vez ao shell de login da pessoa
      (`$SHELL -l -i -c`) qual é o `PATH`, com prazo (um `.zshrc` que trava
      não pode travar o app), e junta ao do processo — sem repetir pasta,
      o do processo primeiro, e `~/.local/bin` de reserva quando nem o
      shell responde. No Windows, nada muda.
- [x] Esse `PATH` vai no processo do agente (TUI e janela), na
      verificação do projeto e no git. No Linux, o `Command` procura o
      executável no `PATH` dado ao filho: o `claude` passa a ser achado.
- [x] Programa que nem assim aparece: a mensagem diz que não está no
      `PATH` do app nem no do shell, e manda pôr o caminho completo
      (`command -v claude`) no agente.
- [x] Testes das partes puras (ler a saída do shell com o lixo que um
      `.zshrc` imprime; juntar sem repetir) e um de verdade: um executável
      numa pasta que só o "shell de login" conhece é achado.
- [x] Conferido como o menu abre: `env -i` com o `PATH` da sessão, a TUI
      subindo o agente falso que mora fora do `PATH`.

## Comandos de validação

```bash
cargo test --workspace
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
env -i HOME=$HOME SHELL=/usr/bin/zsh PATH=/usr/local/bin:/usr/bin ./target/debug/deps/anotadinho_vault-* medir_o_path_como_o_menu_abre --ignored --nocapture
```

## Não-objetivos

- Reinstalar o pacote: é o `scripts/build.sh` da pessoa.
