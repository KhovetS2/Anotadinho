---
id: "501"
titulo: "Projeto na TUI — as execuções do projeto (`x`) e a dica das transições"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["478"]
estima_min: 60
---

# 501 — Execuções do projeto na TUI

Paridade com a aba Execuções da `ProjetoView` da janela
(`ui/src/components/projeto_view.rs`, filtro pela pasta do projeto).

## Critérios de aceite

- [x] `x` na tela do projeto lista as execuções do agente cujas conversas
      moram na pasta do projeto (`projeto::pasta_do_projeto`), como a
      janela. As de outro projeto e as conversas soltas ficam de fora.
- [x] Enter numa execução abre a conversa dela; `r` roda de novo (o
      registro de execuções de sempre, `AcaoDaEscolha::Execucoes`).
- [x] Sem execução no projeto, o aviso diz "nenhuma tarefa executada
      ainda" em vez de abrir lista vazia.
- [x] A dica do rodapé deixa de dizer "1-3": a faixa vem das transições
      da tarefa selecionada (bloqueada tem 4; executando, 1; sem tarefa,
      nenhuma).
- [x] Os atalhos (`?`) e `docs/paridade-gui-tui.md` falam do `x`.

## Validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
python3 scripts/tui/tela.py --abrir meu-primeiro-projeto/projeto --execucoes reg.jsonl --teclas x
```

## Não-objetivos

- Mexer na janela.
- Um registro de execuções por projeto em disco: o filtro é sobre o
  `execucoes.jsonl` de sempre.
