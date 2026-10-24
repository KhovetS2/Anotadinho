---
id: "498"
titulo: "TUI: entrar no bloco de diagrama ou código e rolar nas duas direções"
status: done
criado: 2026-10-24
autor: agente
prioridade: alta
depende_de: ["497"]
estima_min: 240
---

# 498 — Navegar dentro do bloco

Pedido direto da pessoa depois do 497: o diagrama pode passar do tamanho
do painel, e aí ele era cortado embaixo ("… mais N") ou, mais largo que o
painel, virava código com o motivo. Ao selecionar o bloco, deve dar pra
ENTRAR nele e andar pelo desenho na vertical e na horizontal.

## Critérios de aceite

- [x] Enter num bloco de código ou mermaid entra no bloco: a caixa acende,
      o topo diz "navegando" e onde se está (linhas e colunas à vista de
      quantas há), e as teclas passam a mover a vista, não o cursor.
- [x] `j k ↑ ↓` rolam uma linha; `h l ← →` rolam colunas; `Ctrl+d`/`Ctrl+u`
      e `PgDn`/`PgUp` meia tela; `g`/`G` topo e fim; `0`/`$` começo e fim
      da linha. A vista nunca passa do fim nem fica antes do começo.
- [x] Dentro, `i` ou Enter abre o editor do código (o de 390); `Esc` ou
      `q` saem e devolvem o cursor ao bloco.
- [x] Diagrama mais largo que o painel sai DESENHADO na largura dele e
      recortado na borda, com o aviso de quantas colunas tem e "Enter
      navega" — não mais o código.
- [x] Na vista, o código não é cortado com `…`: a linha inteira está lá,
      e a vista desliza por ela.
- [x] `i`/`A` fora da vista continuam abrindo o editor direto.
- [x] Trocar de página, recarregar ou o bloco sumir tira da vista sem
      quebrar.
- [x] Testes com `TestBackend` e a tela real num pty (`scripts/tui/tela.py`).

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test -p anotadinho-tui
cargo test --workspace
python3 scripts/tui/tela.py pages/exemplos/diagrama-mermaid.md --largura 60 --altura 24 --teclas "…"
```

## Não-objetivos

- Rolar com a roda do mouse dentro do bloco (a roda continua rolando a
  página).
- Busca dentro do bloco.
