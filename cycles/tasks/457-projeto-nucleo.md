---
id: "457"
titulo: "Módulo Projeto — núcleo: projeto, tarefa, etapas e prompts"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: []
estima_min: 180
---

# 457 — O núcleo do módulo Projeto

Primeiro dos seis ciclos da proposta
`VaultAnotadinho/pages/propostas/modulo-projeto.md` (spec
`pages/specs/modulo-projeto.md`). Este ciclo é só o modelo puro em
`crates/core/src/projeto.rs`, sem IO. O IO entra no 458, a execução no
459 e as telas no 460 e 461.

## Objetivo

Ter num só lugar, testável e sem IO, o que GUI, TUI e CLI vão
compartilhar:

- o que é um projeto e o que é uma tarefa;
- onde cada página mora;
- quais etapas existem e quem pode mover uma tarefa até cada uma;
- o que entra no contexto da conversa do projeto;
- os prompts de detalhar e de executar;
- como ler o resultado da execução.

## Critérios de aceite

- [x] `EtapaTarefa` tem as oito etapas da spec, com `slug`, `label`,
      `from_slug`, `proximas` e `movida_por`. Uma tarefa nunca vai de
      `ideia` direto pra `executar`.
- [x] `pode_ir_para(destino, quem)`: o humano não move para
      `executando` nem para `revisao`, e o sistema só move para
      `executando`, `revisao` e `bloqueada`.
- [x] `mudar_etapa(texto, destino, quem)` grava o `status` no
      frontmatter pelo `MarkdownCodec` e deixa o corpo byte a byte
      intocado. Uma transição inválida devolve erro dizendo para onde
      a tarefa pode ir.
- [x] `montar_tarefa` e `montar_projeto` geram o frontmatter por
      `serde_yaml`. Um título com `: ` continua sendo YAML válido.
- [x] `Tarefa::ler` e `Projeto::ler` só aceitam o `type` certo.
- [x] `proximo_numero` e `caminho_da_tarefa` numeram sem colidir.
- [x] `fila`: só tarefas em `executar` cujas dependências estão
      `feita`, em ordem de prioridade e depois de número.
- [x] `ler_resultado`: sem a linha `RESULTADO:`, a tarefa vai para
      `bloqueada`, não para `revisao`.
- [x] `separar_tarefas` quebra uma resposta em várias tarefas, no
      formato que o prompt semeado pede.
- [x] A semente cria `pages/projetos` e o prompt
      `quebrar-em-tarefas.md`.

## Decisões deste ciclo (desvios da proposta, apontados)

1. **Onde o projeto mora: `pages/projetos/<slug>/`, e não
   `projetos/<slug>/`.** Todo o resto do vault está sob `pages/`
   (`Artefato::pasta`, `contexto`, a sidebar). Uma raiz nova seria a
   única exceção, sem ganho nenhum.
2. **A fila é por nível, com reserva. Não existe `projetos-visto.json`.**
   Escrevendo `transicoes`, apareceu que a memória por borda PERDE
   pedidos: uma tarefa posta em `executar` com a fila pausada era
   marcada como "vista" e nunca mais rodava depois de retomar a fila. O
   que impede rodar duas vezes já é a reserva (`executar` →
   `executando`, gravada antes de começar). A objeção ao
   `Quando::Consulta` era disparar a cada minuto, e ela não se aplica:
   depois da reserva, a tarefa sai da consulta. Então a fila é
   simplesmente "o que está em `executar`". A emenda na proposta vai
   por `propor`.

## Comandos de validação

```bash
cargo test -p anotadinho-core projeto
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- IO, config `.anotadinho/projetos.json` e CLI (ciclo 458)
- Rodar o agente (459), telas (460/461)
- Explicar `projeto`/`tarefa` no guia da semente: só depois que as telas
  existirem, para o guia não prometer o que ainda não abre
