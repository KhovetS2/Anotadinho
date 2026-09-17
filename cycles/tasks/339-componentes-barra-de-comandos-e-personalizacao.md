---
id: "339"
titulo: "Componentes da TUI, barra de comandos, personalização e opções da seleção"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["338"]
---

# 339 — Componentes, barra de comandos e personalização

- `componentes.rs`: `Campo` (texto de uma linha com cursor), `Lista` (filtro
  sem acento, do mais parecido pro menos, seleção, menu com `j`/`k`),
  `desenhar_modal` (caixa arredondada por cima, título e rodapé com as
  teclas) e `desenhar_lista`.
- `app/modais.rs`: `Modal` (paleta, escolha, confirmação, entrada, atalhos,
  opções) e `Pedido` — o que só o `main` faz (abrir, criar, apagar,
  diário de hoje, gravar preferências).
- Barra de comandos (`:` ou `Ctrl+K`), como a `CommandPalette` da janela:
  nova conversa, nova página (e por tipo), alternar/escolher tema, alternar
  sidebar, hoje, personalizar, trocar agente, atalhos, excluir a página;
  depois as páginas do vault.
- Personalização: tema, sidebar e agente, gravados em
  `~/.config/anotadinho/tui.json` (`--tema` ainda vale e grava por cima).
- `?` abre os atalhos por assunto.
- Opções da seleção: `Enter` no cabeçalho de uma coluna seleção/tags abre o
  editor — `o` cria, `a` renomeia (as células acompanham), `dd` apaga (sai
  das células), `J`/`K` reordenam, `u` desfaz.
- O laço do `main` redesenha a cada 250ms sem tecla (`app::tique`), pra o
  que roda por fora aparecer.
