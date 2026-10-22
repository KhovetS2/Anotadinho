---
id: "467"
titulo: "TUI: a árvore de pastas não 'abria' — a sidebar não rolava; clique e h/l pela base de navegação"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["466"]
estima_min: 150
---

# 467 — A sidebar da TUI rola, aceita clique, e `h`/`l` seguem o modelo de navegação

Relato da pessoa: "na TUI a árvore de pastas não está abrindo".

## O que foi medido (pty com `pyte`, numa cópia do vault)

- **Com 40 linhas, pelo teclado, a árvore abre.** `projetos`,
  `anotadinho` e `tarefas` abrem, e as 4 tarefas aparecem dentro.
- **A sidebar NÃO ROLA.** É um `Paragraph` sem `scroll`. Num terminal de
  24 linhas, abrir `specs/` empurra o resto da lista pra baixo da borda,
  e o cursor anda até lá sem aparecer. Uma pasta cujo conteúdo nasce
  abaixo da borda parece não ter aberto. O módulo Projeto piorou isso:
  `projetos/<slug>/tarefas` são três níveis a mais na lista.
- **O clique do mouse é ignorado** de propósito desde o 454 ("a TUI é de
  teclado"). Quem vem da janela abre pasta clicando, e na TUI nada
  acontece.
- **`h` e `l` faziam a MESMA coisa:** alternavam a pasta. `h` numa pasta
  fechada a ABRIA. O modelo de `core::navegacao` diz que entrar e sair
  são atos distintos, e que na borda o cursor fica.

## Critérios de aceite

- [x] A sidebar rola e o cursor está SEMPRE visível. Abrir uma pasta
      mostra o que entrou nela, tanto quanto cabe, sem tirar o cursor da
      tela.
- [x] Clique: numa pasta, abre ou fecha; numa página, abre a página.
      O foco vai pra onde o clique caiu.
- [x] `l`/`→`: numa pasta fechada, abre; numa pasta aberta, entra no
      primeiro filho; numa página, não faz nada (é a borda).
- [x] `h`/`←`: numa pasta aberta, fecha; numa pasta fechada ou numa
      página, sobe pra pasta que a contém; na raiz, não faz nada.
- [x] Testes de unidade, e o pty a 24 linhas abrindo uma pasta perto do
      fim.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
# pty: scratchpad/tui467/tela.py (pyte), ALTURA=24
```

## Não-objetivos

- Arrastar página entre pastas com o mouse (`m` move)
