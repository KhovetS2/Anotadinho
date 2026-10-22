---
id: "459"
titulo: "Módulo Projeto — a tarefa em executar roda o agente no repositório do projeto, na janela e na TUI"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: ["458"]
estima_min: 240
---

# 459 — Executar no repositório do projeto

Terceiro ciclo da proposta `pages/propostas/modulo-projeto.md`. Liga a
fila do 458 aos dois executores de agente que já existem: o da janela
(`src-tauri`) e o da TUI.

## Critérios de aceite

- [x] A janela confere a fila dos projetos a cada 5 s (comando
      `avaliar_projetos`), e a TUI no próprio laço, no mesmo ritmo.
- [x] O agente de uma tarefa roda com o `cwd` no repositório do projeto,
      que entra por PARÂMETRO (`Adaptador::cwd`), sem `cfg!` nem estado
      global.
- [x] O `ipc` nunca reserva mais tarefas do que o frontend tem vaga pra
      iniciar (`ParametrosDaFila::vagas`). Uma reserva sem vaga deixaria
      a tarefa presa em `executando`.
- [x] Ao terminar, por qualquer caminho (resposta, falha, interrupção ou
      falha ao subir, inclusive saindo da fila global), a tarefa sai de
      `executando` por `handle_fechar_tarefa_da_conversa`.
- [x] Com a conversa da tarefa já em sessão (`--resume`), o histórico não
      é repetido no prompt, como a TUI já faz na conversa comum.
- [x] Os avisos da fila (sem repositório, pausada, esperando dependência)
      aparecem uma vez quando mudam, não a cada 5 s. Na TUI eles vão pro
      rodapé. A janela não tem superfície global de aviso, então eles vão
      pro console, e a tela do 460 passa a mostrá-los no cabeçalho do
      projeto.
- [x] Ponta a ponta com um agente falso de verdade, um executável que
      grava o `pwd` e responde `RESULTADO: concluida`. A tarefa vai de
      `executar` a `revisao`, o arquivo aparece no repositório, e a
      resposta e o registro de execução existem. Feito na janela (pelo
      harness) e na TUI (num pty).

## Decisões

- **Sem fila por projeto no frontend.** A proposta previa um `Fila` por
  projeto. Não é preciso: a concorrência por projeto já é decidida pelo
  vault no 458, e a fila global de cada frontend continua limitando o
  total de processos. Duas filas aninhadas seriam dois lugares
  decidindo a mesma coisa.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
node scripts/uitest/run.mjs projeto
```

## Não-objetivos

- A tela de projeto (460/461). Aqui a tarefa é movida pelo CLI ou pela
  edição do arquivo.
- Cancelar o agente quando a pessoa move a tarefa pra `bloqueada` durante
  a execução. O fechamento já respeita a decisão dela, mas o processo
  segue até o fim. Fica anotado pro 460, que tem o botão.
