---
id: "513"
titulo: "Projeto: editar prioridade e dependências da tarefa (núcleo, ipc e janela)"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["477"]
estima_min: 120
---

# 513 — Prioridade e dependências

A fila já ordenava por `prioridade` e segurava por `depende` (ciclos 458,
462), mas nem a janela nem a TUI deixavam mudar os dois: só editando o
frontmatter à mão, onde "urgente" cai em média e `depende: 3, 4` vira
outra coisa sem aviso. Achado no inventário de paridade do projeto.

## Critérios de aceite

- [x] Núcleo:
  - [x] `projeto::com_prioridade` aceita só alta/media/baixa, e `None`
        tira o campo;
  - [x] `com_dependencias` grava lista YAML sem repetidas nem vazias e
        tira o campo quando fica vazia;
  - [x] uma tarefa não pode depender de si, e quem não é tarefa é
        recusado;
  - [x] o corpo fica intocado.
- [x] `ipc`: `handle_prioridade_da_tarefa` e
      `handle_dependencias_da_tarefa`, pelo mesmo `ler_tarefa` dos outros
      detalhes; e os comandos Tauri.
- [x] Janela, no painel de detalhes:
  - [x] o seletor de prioridade;
  - [x] as dependências como chips com o título e o x pra tirar;
  - [x] "+ depende de…" com as OUTRAS tarefas do projeto.
- [x] O cartão ganha o selo ↑ alta / ↓ baixa; média não aparece.
- [x] Cenário (513): pôr e tirar as duas coisas, com o cartão e o
      arquivo conferidos.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib projeto
cargo test --workspace
node scripts/uitest/run.mjs "(513)"
node scripts/uitest/run.mjs "(477)"
```

## Não-objetivos

- Ordenar o quadro pela prioridade: a coluna continua na ordem do
  número, e a fila é que usa a prioridade.
- Dependência entre projetos.
