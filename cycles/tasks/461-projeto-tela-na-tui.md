---
id: "461"
titulo: "Módulo Projeto — a tela de Projeto na TUI"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: ["460"]
estima_min: 240
---

# 461 — A tela de Projeto na TUI

Quinto ciclo da proposta `pages/propostas/modulo-projeto.md`. É a mesma
tela da janela (460), no terminal. A regra continua no núcleo e o IO no
`ipc`: a TUI só desenha e manda pedidos.

## Critérios de aceite

- [x] Uma página `type: projeto` abre a tela de Projeto:
  - [x] Cabeçalho com o repositório, a fila (ativa ou pausada), o que
        está executando e os avisos.
  - [x] As tarefas agrupadas por etapa, na ordem do quadro.
  - [x] Na tarefa selecionada, as transições humanas válidas, numeradas.
        O número move a tarefa.
- [x] Teclas:
  - [x] `j`/`k` passam pelas tarefas.
  - [x] `1`…`9` movem a tarefa pela transição daquele número.
  - [x] `Enter` abre a tarefa.
  - [x] `n` cria uma tarefa.
  - [x] `d` detalha: abre a conversa da tarefa com o pedido escrito.
  - [x] `c` abre a conversa do projeto, `a` a arquitetura.
  - [x] `p` pausa ou retoma a fila, `o` define o repositório.
- [x] A tela se atualiza sozinha (~3 s): é o agente que move as tarefas
      pra `executando` e `revisao`.
- [x] Mover pela tela dispara a checagem da fila NA HORA, sem esperar a
      volta de 5 s.
- [x] Bloquear uma tarefa em `executando` interrompe o agente dela
      nesta TUI.
- [x] Paleta: "Novo projeto…" e "Mover a tarefa aberta…" (na página de
      uma tarefa).
- [x] Conversa de projeto:
  - [x] o índice das tarefas vai como contexto;
  - [x] a resposta ganha "virar tarefas" e "atualizar arquitetura", que
        propõem.
- [x] Validado num pty: abrir o projeto, mover uma tarefa pelo número e
      ver o disco mudar, e criar uma tarefa.

## Decisões

- **Lista agrupada por etapa, não colunas.** Num terminal de 140
  colunas, 8 colunas dão ~16 caracteres por cartão, o que corta todo
  título. O kanban da TUI usa colunas porque tem 4. Aqui, cada etapa vira
  uma seção, e as vazias aparecem como uma linha só.
- **Transição por número, não por menu.** São no máximo três por etapa,
  sempre visíveis embaixo da tarefa selecionada: uma tecla em vez de
  abrir, escolher e confirmar.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo test --workspace
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
# pty: scripts no scratchpad, descritos no status
```

## Não-objetivos

- Barra de etapa DENTRO do editor da TUI. A paleta "Mover a tarefa
  aberta…" cobre o gesto sem mexer no desenho do documento.
