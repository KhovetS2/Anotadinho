---
id: "460"
titulo: "Módulo Projeto — a tela de Projeto e a barra de etapa da tarefa, na janela"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: ["459"]
estima_min: 300
---

# 460 — A tela de Projeto na janela

Quarto ciclo da proposta `pages/propostas/modulo-projeto.md`.

## Critérios de aceite

- [x] "Novo projeto" na paleta pede o título e cria o projeto. A página
      do projeto abre logo depois.
- [x] Uma página `type: projeto` abre a tela de Projeto:
  - [x] Cabeçalho com o repositório, escolhido no seletor de pasta, o
        botão pra pausar e retomar a fila, e os avisos da fila (que até
        o 459 só iam pro console).
  - [x] Quadro com uma coluna por etapa. Cada cartão mostra número,
        título e dependências, e SÓ os botões das transições humanas
        válidas (`proximas_para(Humano)`).
  - [x] Criar tarefa pelo próprio quadro.
  - [x] Abas Quadro, Conversa e Execuções. A Conversa é a `ConversaView`
        da conversa geral, e as Execuções são as do registro, filtradas
        pelas conversas do projeto.
  - [x] Atalho pra abrir a arquitetura.
- [x] Uma página `type: tarefa` abre o editor de sempre, com uma barra de
      etapa em cima:
  - [x] a etapa atual e as transições válidas;
  - [x] "Detalhar com o agente", que abre a conversa do projeto com a
        pergunta de detalhar já escrita, sem enviar;
  - [x] "Conversa da execução", quando ela existe.
- [x] A conversa de um projeto (com `projeto:` no frontmatter) leva o
      índice das tarefas como contexto e ganha, nas respostas:
  - [x] **virar tarefas**: um lote de propostas, uma por
        `## Tarefa:`, numeradas sem colidir;
  - [x] **atualizar arquitetura**: uma proposta sobre `arquitetura.md`.
  - Os dois PROPÕEM, e a pessoa escolhe na revisão.
- [x] Mover pra `bloqueada` uma tarefa em `executando` interrompe o
      agente dela.
- [x] DOM novo com `data-nav-item`/`data-nav-group`, CSS com tokens e em
      BEM, ícone novo em `icon.rs`.
- [x] Harness: criar pela paleta, o quadro com só as transições válidas,
      mover etapa pelo cartão, virar tarefas e a barra da tarefa.

## Decisões

- **Arquitetura abre como página, não como aba com editor.** O `Editor`
  tem vinte props ligadas ao app (vim, autosave, navegação por bloco).
  Embuti-lo numa aba duplicaria essa fiação pra ganhar um clique.
- **Virar tarefas propõe, e virar spec grava.** O "virar spec" já existia
  e grava direto, porque é uma página só e quem clicou leu. "Virar
  tarefas" cria várias de uma vez, e o prompt semeado diz "eu escolho
  quais viram tarefa". O lote na revisão é esse "escolher".

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs projeto
node scripts/uitest/run.mjs
```

## Não-objetivos

- Arrastar cartão entre colunas: os botões já cobrem, e arrastar é o
  caminho que mais custou no editor
- Tela da TUI (461)
