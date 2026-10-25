---
id: "505"
titulo: "TUI: diagrama alto sai pela metade em vez de sumir; sequência com as caixas embaixo; a roda rola dentro do bloco"
status: done
criado: 2026-10-24
autor: agente
prioridade: alta
depende_de: ["498", "504"]
estima_min: 90
---

# 505 — "Esse diagrama cortou e não está rolando"

A pessoa comparou a página `pages/exemplos/diagrama-mermaid.md` na janela
e na TUI. Reproduzi o relato no pty e achei três causas.

1. **O diagrama da sequência parecia cortado.** O mermaid repete as
   caixas dos participantes embaixo (`mirrorActors`, o padrão), e a TUI
   não repetia. As linhas da vida acabavam no vazio.
2. **O fluxograma alto sumia junto com o resto da página.** Um grupo de
   linhas é indivisível: se não cabia no que sobrava da tela, ele e tudo
   depois dele ficavam pro próximo quadro. A página parava em
   "## Fluxo de uma tarefa", em branco embaixo.
3. **A roda não rolava o diagrama.** Ela anda o cursor entre blocos, e no
   último bloco da página não fazia nada, mesmo dentro do bloco (498).

## Critérios de aceite

- [x] A sequência desenha as caixas dos participantes também embaixo, com
      `┴` onde a linha da vida chega.
- [x] Um bloco de código ou diagrama mais alto que o espaço que sobra sai
      pela metade, com "┆ ⋯ mais N linha(s) abaixo", e o que vem depois
      continua acessível. Com o cursor nele, a tela rola até ele caber
      inteiro (a regra do 504 continua).
- [x] Dentro do bloco, a roda do mouse rola o desenho de 3 em 3 linhas.
- [x] Testes vistos reprovando com cada correção desligada.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test -p anotadinho-tui
python3 scripts/tui/tela.py VaultAnotadinho/pages/exemplos/diagrama-mermaid.md --largura 150 --altura 30
```

## Não-objetivos

- O rótulo da mensagem que cruza outra linha da vida: na janela ele
  passa por cima da linha, e aqui ele para antes dela.
