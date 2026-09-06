---
id: "276"
titulo: "Passo 4a: a gravação preserva o que ninguém tocou"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["275"]
estima_min: 240
---

# 276 — A gravação preserva o que ninguém tocou

## O que é, dentro do passo 4

A primeira fatia da inversão que chega ao ARQUIVO. A árvore ainda não é
a fonte da verdade do editor, mas passa a ser **quem decide o que
mudou** na hora de gravar.

Hoje o corpo que sai do DOM é uma reconstrução inteira: todo embed é
reserializado e todo markdown volta pela travessia do HTML. Isso muda
bytes que ninguém tocou — ordem de campos de YAML, espaço no fim de
linha (quebra forte em markdown), recuo de continuação.

## Critérios de aceite

- [x] `costurar_mudancas` decide pela ÁRVORE o que mudou
- [x] O alinhamento tolera mudança de estrutura (bloco inserido,
      apagado, partido em dois)
- [x] Diferença só de espaço branco não conta como edição — mas dentro
      de código conta
- [x] O cenário foi visto REPROVANDO com a costura desligada
- [x] O defeito que sobra fica afirmado, não tolerado

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
