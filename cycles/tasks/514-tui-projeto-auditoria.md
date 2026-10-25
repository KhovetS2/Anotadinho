---
id: "514"
titulo: "TUI: prioridade e dependências da tarefa, e a auditoria de desenho e navegação do Projeto"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["513"]
estima_min: 180
---

# 514 — O Projeto na TUI, conferido tela por tela

Pedido: "uma verificação na TUI se nela está implementada todas as
funcionalidades do projeto com desenhos bonitinhos e boa navegabilidade".

Fiz a auditoria no terminal de verdade:
- capturei com cor, em 150×30 e 80×24, o quadro, os detalhes, as
  sprints, o filtro, as execuções e a página da tarefa;
- converti as capturas pra imagem pra olhar o desenho.

Daí saíram, além da paridade do 513, cinco defeitos de navegação.

## Critérios de aceite

- [x] Paridade do 513:
  - [x] `v` ganha "Prioridade: …", que abre a escolha entre média, alta
        e baixa e volta pros detalhes;
  - [x] "Depende de: …" lista as OUTRAS tarefas com ■/□, e Enter liga e
        desliga sem fechar;
  - [x] o cartão ganha ↑alta/↓baixa;
  - [x] os handlers são os do 513.
- [x] **Barra de comandos:** a página tem o título do frontmatter (◆ projeto,
      ☐ tarefa). Antes, "Meu primeiro projeto" não achava o projeto, que
      se chamava "projeto" como todos.
- [x] **Rodapé:** cabe na largura e mantém sempre o "? atalhos". Em 80
      colunas ele era cortado no meio.
- [x] **Transições:**
  - [x] saem inteiras, sem "2 → D";
  - [x] quebram por opção numa tela estreita;
  - [x] a faixa tem altura fixa pela largura, pro quadro não pular ao
        mudar a seleção.
- [x] **Cabeçalho:**
  - [x] o aviso quebra em vez de cortar;
  - [x] a linha da sprint ganha `…`;
  - [x] o repositório vazio sai "nenhum · o escolhe", e não mais
        "repositório sem repositório".
- [x] **Clique no cartão:** seleciona, e de novo abre, como na janela.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
python3 scripts/tui/tela.py --abrir "Meu primeiro projeto" --largura 80 --altura 24 --teclas l
python3 clique.py   # o clique SGR mandado pelo tmux: 1 seleciona, 2 abre
```

## Não-objetivos

- Arrastar o cartão entre colunas pelo mouse: os números continuam sendo
  o caminho (465).
- O nome da página na sidebar continua o do arquivo.
