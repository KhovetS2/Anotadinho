---
id: "503"
titulo: "Sprints na TUI — criar com objetivo e datas, o histórico com as feitas, e o comentário inteiro"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["502"]
estima_min: 120
---

# 503 — Sprints com datas e histórico, e o comentário inteiro

Paridade com a aba Sprints da janela (`ui/src/components/sprints_do_projeto.rs`)
e com o painel de detalhes da tarefa, que mostra o comentário inteiro.
Na TUI a sprint nova nascia só com o nome (`String::new(), None, None`),
o histórico era leitura muda e o comentário aparecia só na 1ª linha.

## Critérios de aceite

- [x] "Nova sprint…" (em `S`) abre um formulário: nome, objetivo, início
      e fim. As datas usam o seletor de data dos formulários (ciclo 364:
      Enter abre, `c` digita); digitadas, valem só como `AAAA-MM-DD`, e o
      fim não vem antes do início. Sem nome, não cria. O
      `handle_criar_sprint` recebe objetivo e datas.
- [x] O menu `S` mostra, de cada sprint, as datas (como a janela:
      "a → b", "desde a", "até b") e o objetivo.
- [x] Uma sprint encerrada do histórico se escolhe: lista as tarefas
      FEITAS nela (as de `PorSprint`, como a janela), e Enter numa abre a
      tarefa.
- [x] Nos detalhes da tarefa (`v`), Enter num comentário abre o texto
      inteiro, com quebra de linha e rolagem; Esc volta pros detalhes.
- [x] Atalhos e `docs/paridade-gui-tui.md` atualizados.

## Validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
python3 scripts/tui/tela.py --abrir meu-primeiro-projeto/projeto --teclas "S j j j j j Enter …"
```

## Não-objetivos

- Editar o objetivo ou as datas de uma sprint que já existe (a janela
  também não edita: é a página da sprint).
- Mexer na janela.
