---
id: "504"
titulo: "TUI: bloco que não cabe na borda de baixo rola em vez de sair como moldura vazia"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["500"]
estima_min: 45
---

# 504 — Bloco na borda de baixo

Achado no ciclo 500: um bloco de código (ou tabela) inserido no fim da
tela aparece como uma moldura vazia até a próxima tecla. Com o menu `/` →
Código, isso acontece numa tela de 110×20.

## Causa

O `montar` da tela marcava o cursor como "achado" assim que chegava à
linha dele, antes de saber se o grupo daquela linha cabia. Um grupo é
indivisível: ou entra inteiro, ou fica pro próximo quadro. Quando o bloco
não cabia, ficava só a moldura do foco. A tela achava que o cursor estava
visível, então a rolagem que corrige isso não acontecia.

## Critérios de aceite

- [x] O cursor só conta como visível se o primeiro grupo da linha dele
      coube.
- [x] Teste: um bloco de 3 linhas como último bloco de uma página de 14
      parágrafos, numa tela de 20 linhas, aparece inteiro. Visto
      reprovando sem a correção.
- [x] No pty, `G o /` → Código aparece com o bloco, e não com a moldura
      vazia.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
python3 scripts/tui/tela.py borda-de-baixo.md --largura 110 --altura 20 --teclas 'G o / Down Down Down Down Down Down Enter'
```

## Não-objetivos

- Um grupo mais alto que o painel inteiro: esse já tem teto (494, 498).
