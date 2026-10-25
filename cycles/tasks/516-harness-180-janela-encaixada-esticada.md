---
id: "516"
titulo: "Harness: o (180) reconhece a janela encaixada que o gerenciador estica ao restaurar"
status: done
criado: 2026-10-25
autor: agente
prioridade: baixa
depende_de: ["455"]
estima_min: 20
---

# 516 — A segunda cara do encaixe no (180)

Depois do 513, o app de desenvolvimento reiniciou (o `src-tauri` mudou) e
o Hyprland o encaixou em meia tela. O (180) passou a reprovar sempre, com
"o estado devolvido tem que bater com o real".

## Medido

Uma sonda chamou `window_is_maximized` antes e depois de
`window_toggle_maximize`:

| Momento | Maximizada? | Tamanho |
|---|---|---|
| Antes | sim | 913×1048 |
| Pedido de restaurar | volta `false` | — |
| Depois | sim | 1838×1048 |
| Na volta | sim | 913×1048 |

O GTK chama de "maximizada" a janela encaixada. O gerenciador responde
ao restaurar esticando-a, e o estado não muda. É o mesmo encaixe do
ciclo 455, mas com o tamanho MUDANDO, então a regra de lá ("estado e
tamanho iguais") não o pegava.

O 513 só acrescentou dois comandos ao `main.rs`, sem tocar na janela.

## Critérios de aceite

- [x] Começou "maximizada", a largura é menos de 90% da tela e o estado
      não bateu: o cenário diz "não se aplica", com os tamanhos medidos.
- [x] O resto das asserções continua valendo numa janela que não está
      encaixada.

## Comandos de validação

```bash
node scripts/uitest/run.mjs "(180)"
```
