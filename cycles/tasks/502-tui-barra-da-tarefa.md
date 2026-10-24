---
id: "502"
titulo: "Página de tarefa na TUI — a barra da tarefa sobre o editor"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["501"]
estima_min: 90
---

# 502 — A barra da tarefa na TUI

Paridade com a `TarefaBarra` da janela (`ui/src/components/tarefa_barra.rs`,
montada em cima do editor pelo `page_view.rs` nas páginas `type: tarefa`).
Hoje a TUI só tem "Mover a tarefa aberta…" na barra de comandos.

## Critérios de aceite

- [x] Numa página `type: tarefa`, em cima do conteúdo, uma faixa com: o
      projeto, o número e o título, a etapa atual e as transições da
      pessoa (`EtapaTarefa::proximas_para(Quem::Humano)`), e uma linha de
      dica com as teclas. Fora de tarefa, nada muda no editor.
- [x] As ações da barra, pela barra de comandos E por tecla:
  - [x] mover de etapa (a escolha de sempre, `handle_mudar_etapa_tarefa`);
  - [x] "Detalhar com o agente": a conversa da tarefa com a
        `pergunta_de_detalhar` escrita no campo, sem enviar — só em ideia,
        detalhando e bloqueada, como a janela;
  - [x] "Conversa da tarefa": a mesma conversa, sem pergunta
        (`handle_conversa_da_tarefa`);
  - [x] abrir o quadro do projeto.
- [x] As teclas não brigam com o vim: `Alt+e` / `Alt+a` / `Alt+c` /
      `Alt+p`, como atalhos globais remapeáveis ("Remapear teclas").
- [x] Mover a etapa com edição ainda não salva grava a edição ANTES —
      senão a releitura da página jogava o texto fora.
- [x] A rolagem do editor conta a faixa: o cursor nunca fica escondido
      atrás dela.
- [x] Atalhos (`?`) e `docs/paridade-gui-tui.md` atualizados.

## Validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
python3 scripts/tui/tela.py --abrir 002-cadastrar-um-livro --teclas "M-e"
```

## Não-objetivos

- Mexer na janela.
- Clicar na faixa com o mouse: a TUI só clica na sidebar.
