---
id: "495"
titulo: "TUI: cor por linguagem no bloco de código"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["494"]
estima_min: 150
---

# 495 — A cor do código

Segunda etapa da proposta `pages/propostas/tui-codigo-e-diagramas.md`
(RF2, RF3, RNF2, RNF4).

## Critérios de aceite

- [x] Palavra-chave, string, comentário, número e tipo com cor, pelas cores
      do tema da TUI (nos cinco temas, sem cor de socorro).
- [x] A lista mínima da spec reconhecida pelos nomes de cerca comuns: Rust,
      JS/TS, Python, shell, JSON, YAML, TOML, HTML, CSS, SQL e Markdown.
- [x] Linguagem desconhecida ou ausente: uma cor só, a do código.
- [x] As gramáticas carregam na primeira vez que um bloco com linguagem é
      desenhado; o realce de cada bloco é guardado.
- [x] Escala: numa página de 400 blocos, só os da tela passam pelo parser,
      e redesenhar não passa de novo. Visto reprovando sem o guardado.
- [x] `cargo check` no alvo do Windows.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo test --workspace
PATH=~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH \
  cargo check -p anotadinho-tui -p anotadinho-cli --target x86_64-pc-windows-gnu
python3 scripts/tui/tela.py pagina-com-codigo.md --cores
```

## Não-objetivos

- Diagramas (496–497).
