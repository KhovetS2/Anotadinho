---
id: "515"
titulo: "TUI: trocar texto no editor de código (este ou todos), e a busca que achava o 2º em vez do 1º"
status: done
criado: 2026-10-25
autor: agente
prioridade: baixa
depende_de: ["509"]
estima_min: 45
---

# 515 — Trocar no editor de código

O que o 509 deixou de fora: depois de achar, trocar.

## Critérios de aceite

- [x] Na busca (Ctrl+F), Tab passa pro campo "trocar por" e volta.
- [x] Alt+R troca o achado do cursor e vai ao próximo; Alt+A troca todos
      num passo só do desfazer, e o topo diz quantas.
- [x] Sem diferença de maiúscula, como a busca.
- [x] **Defeito do 509:** com o cursor no começo do texto, Enter pulava o
      primeiro achado (o `cursor - 1` não recuava). Agora fica nele.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
python3 scripts/tui/tela.py trocar-no-editor.md --teclas 'j i C-f f o o Tab b a r Enter M-a'
```

## Não-objetivos

- Expressão regular.
