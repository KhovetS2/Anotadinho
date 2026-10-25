---
id: "517"
titulo: "Projeto: o repositório no fluxo da tarefa — verificar, o que mudou, branch e commit (núcleo, ipc e CLI)"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["459", "514"]
estima_min: 240
---

# 517 — O repositório entra no fluxo da tarefa

Pedido: deixar o Anotadinho apto pra construir um SaaS com agentes. A
avaliação apontou que "concluída" era só a palavra do agente, e que a
revisão não mostrava o código. Este ciclo é a base (núcleo, `ipc` e CLI).
A janela e a TUI vêm nos próximos.

## Critérios de aceite

- [x] `ConfigProjeto` ganha quatro campos, todos opcionais e com padrão,
      então um `projetos.json` antigo continua valendo:
  - `verificar` (o comando) e `prazo_da_verificacao` (padrão 900 s);
  - `branch_por_tarefa`;
  - `commitar_ao_aceitar`.
- [x] `ipc::repositorio`:
  - `verificar`: prazo, stdout e stderr, as últimas 60 linhas e o shell
    por parâmetro;
  - `base_atual`, `branch_atual`, `nome_do_branch` e `ir_pro_branch`
    (cria, ou volta pro que já existe);
  - `mudancas_desde`: os commits e a árvore desde a base, mais os
    arquivos novos, com teto de tamanho;
  - `commitar`, em que "nada pra commitar" não é erro.
- [x] Na reserva:
  - com `branch_por_tarefa`, o repositório vai pro branch da tarefa;
  - `base_git` e `branch_git` vão pro frontmatter;
  - a segunda rodada mantém a base da primeira;
  - um `git switch` que falha deixa a tarefa em `executar`, com o
    motivo.
- [x] O prompt diz o comando de verificação, o branch e, na segunda
      rodada, que os Comentários e a Verificação dizem o que mudar.
- [x] No fechamento, com "concluída" do agente:
  - a verificação roda, e se falha a tarefa vai pra `bloqueada`;
  - o relato vai pra seção **Verificação** da tarefa (uma só, a da
    última rodada), e o resultado pra `verificacao:` no frontmatter;
  - a tarefa é relida antes de fechar, porque a pessoa pode ter mexido
    durante a verificação.
- [x] `handle_mudancas_da_tarefa` e `cli tarefa mudancas [--resumo]`.
- [x] Aceitar (revisão → feita) com `commitar_ao_aceitar` commita como
      "tarefa NNN: título" ANTES de gravar a etapa. Se o commit falha, a
      tarefa fica em revisão, com o erro.
- [x] A TUI fecha a tarefa numa thread: a verificação levaria a suíte do
      SaaS pro laço da tela.

## Comandos de validação

```bash
cargo test -p anotadinho-ipc --lib
cargo test -p anotadinho-core --lib projeto
cargo test --workspace
cargo check --target x86_64-pc-windows-gnu  # ipc, tui, cli e src-tauri
```

## Não-objetivos

- Ler a configuração ou alterá-la pela página ou pelo CLI: ela fica com
  a pessoa, como o repositório (padrão "agente e execução"). O agente não
  muda o comando que confere o trabalho dele.
- Merge do branch da tarefa: fica com a pessoa ou com o agente.
