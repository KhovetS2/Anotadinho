---
id: "458"
titulo: "Módulo Projeto — ipc e CLI: criar, listar, mudar etapa, fila com reserva"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: ["457"]
estima_min: 210
---

# 458 — A camada compartilhada do módulo Projeto

Segundo ciclo da proposta `pages/propostas/modulo-projeto.md`, já com a
emenda aprovada: fila por nível, com reserva. O IO fica em
`crates/ipc/src/projeto.rs`, que GUI e TUI vão chamar igual. O CLI
expõe esse IO pro terminal e pro agente.

## Critérios de aceite

- [x] `handle_criar_projeto` cria as três páginas e a pasta de tarefas.
      Rodar de novo não sobrescreve nada e diz o que já existia.
- [x] `handle_listar_projetos` e `handle_tarefas_do_projeto` leem só
      `pages/projetos/*`, sem varrer o vault.
- [x] `handle_mudar_etapa_tarefa` move sempre como `Quem::Humano`.
      Nenhuma função pública deixa um chamador escolher `Sistema`.
- [x] `.anotadinho/projetos.json` guarda `repositorio`, `pausado` e
      `concorrencia` (padrão 1). Arquivo ausente vale o padrão; arquivo
      ilegível dá erro em vez de virar padrão em silêncio.
- [x] `handle_tarefas_a_executar`:
  - [x] Respeita pausa, concorrência e o freio do dia. As tarefas em
        `executando` ocupam vaga.
  - [x] Projeto sem repositório, ou com um caminho que não é pasta, não
        roda e aparece num aviso.
  - [x] Reserva cada tarefa (`executar` → `executando`) antes de
        devolvê-la, relendo do disco sob uma trava de arquivo. Duas
        chamadas seguidas nunca devolvem a mesma tarefa.
  - [x] Cria ou reaproveita a conversa da tarefa, com a pergunta
        gravada antes, e devolve prompt, conversa e `cwd`.
- [x] `handle_fechar_tarefa` leva a tarefa de `executando` para `revisao`
      ou `bloqueada` (pelo `ler_resultado`). Se a pessoa já tiver movido
      a tarefa durante a execução, não mexe.
- [x] CLI:
  - [x] `projeto novo|listar|tarefas` e `tarefa nova|etapa`.
  - [x] Os comandos que escrevem propõem, como `propor` e `etapa`. O
        `projeto novo` vai num lote só.
  - [x] `contexto` lista os projetos com a contagem por etapa.

## Decisões

- **A vaga é contada pelo vault, não pela memória do processo.** Uma
  tarefa em `executando` ocupa vaga, seja qual for o frontend que a
  pegou. Assim GUI e TUI abertas juntas não estouram a concorrência.
  O preço: se o app cair no meio, a tarefa fica em `executando` e
  segura a fila. Ela fica visível, e a pessoa a move pra `bloqueada`.
  É melhor do que rodar duas vezes no mesmo repositório.
- **`fechar` não grava a resposta nem registra a execução.** Os dois
  executores (Tauri e TUI) já fazem isso pra toda conversa. Repetir aqui
  gravaria a resposta duas vezes.
- **Pelo CLI, tudo propõe.** O CLI é o canal do agente. Um `projeto novo`
  ou `tarefa etapa --para executar` gravado direto pularia a revisão. A
  pessoa cria e move pela GUI e pela TUI (ciclos 460 e 461), que chamam
  os `handle_*` diretamente.

- **O teto de profundidade das varreduras subiu de 3 para 8.** Isso não
  estava previsto. `VaultIo` parava em `max_depth(3)`, repetido em cinco
  `WalkDir`. `pages/projetos/<slug>/tarefas/x.md` fica no nível 4, então
  as tarefas e as conversas sumiam da sidebar, da busca, das consultas e
  do `contexto`, sem erro nenhum. Apareceu no teste de ponta a ponta,
  quando o `contexto` disse "sem tarefas" com uma tarefa no disco. O
  número virou `PROFUNDIDADE_MAXIMA`, com teste. A bateria de escala
  continua linear (~20 µs/página).

## Comandos de validação

```bash
cargo test -p anotadinho-ipc projeto
cargo test -p anotadinho-cli projeto
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
cargo check -p anotadinho-cli --target x86_64-pc-windows-gnu
```

## Não-objetivos

- Disparar o agente (459) e telas (460/461)
- Ferramenta MCP nova: o agente cria tarefa com `propor`
