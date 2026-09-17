---
id: "345"
titulo: "Paridade: pastas na sidebar, mover e exportar"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["344"]
---

# 345 — Pastas, mover e exportar

- A árvore da sidebar mostra as pastas do disco, inclusive vazias
  (`handle_list_folders` → `Estado::com_pastas`).
- Na sidebar: `o` página nova na pasta do cursor (ou da página), `O` pasta
  nova dentro dela, `m` mover a página (escolha filtrável das pastas), `dd`
  excluir a página (com confirmação; numa pasta não faz nada).
- Barra: "Nova pasta…", "Mover página pra pasta…", "Exportar pasta…",
  "Exportar vault inteiro" — a exportação é o `handle_export_folder`
  gravado em `anotadinho-<pasta>.md` na pasta de onde a TUI rodou (a
  janela baixa o arquivo).
