---
id: "194"
titulo: "Modos explícitos e teclas por modo"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [175, 193]
estima_min: 180
agente_alvo: claude-opus
---

# Modos explícitos e teclas por modo

## Objetivo

Corrigir uma PERDA DE DADO e tornar explícito o que era implícito:
quais teclas são comandos e quais são texto.

O bug, relatado pelo usuário: digitando uma sequência aleatória no
editor, cada `d` apagava um bloco. Os atalhos de bloco (`d`, `n`, `y`,
`K`, `J`, `c`) dependiam de `bloco_focado()`, que antes do ciclo 175
devolvia `None` durante a digitação — o elemento focado era o CONTÊINER.
Quando o `contenteditable` desceu pro bloco, a distinção sumiu. Ela
nunca deveria ter sido implícita.

## Critérios de aceite

- [x] `Modo` explícito (`Navegacao`, `VimNormal`, `Edicao`), com um
      lugar único que responde "qual modo é este".
- [x] Atalhos de bloco só disparam em `Navegacao`.
- [x] Indicador de modo na barra de baixo, com os atalhos daquele modo.
- [x] Cenários de harness que provam que um comando NÃO dispara no modo
      errado.
- [x] Página sem embed também com `contenteditable="false"` no contêiner.
- [x] Enter quebra linha; Shift+Enter cria bloco; Shift+Enter em bloco de
      código fecha o bloco e abre um parágrafo depois.
- [x] Sem fundo colorido no bloco em foco.
- [x] Dica em bloco vazio: sempre na página vazia, só no hover fora dela.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Modo de seleção múltipla de blocos (é outro ciclo).

## Mudanças de comportamento registradas

A bateria do 193 diz que um cenário dela só muda se o comportamento DEVE
mudar e a mudança estiver escrita na task. Duas mudaram aqui:

1. `Enter no fim cria um parágrafo novo` → `Shift+Enter no fim...`
2. `Enter no meio divide o parágrafo` → `Shift+Enter no meio...`

Motivo: sem isso não havia como quebrar linha DENTRO de um bloco.

## Notas

**A reescrita do 175 estava incompleta** e só apareceu aqui: existem
DOIS caminhos de renderização no editor — com embeds e sem embeds. Só o
primeiro tinha recebido `contenteditable="false"`. Numa página sem
embed ficavam dois editáveis aninhados (contêiner E bloco), que é o que
fazia o Enter num bloco vazio criar parágrafo no lugar errado e o bloco
de origem crescer junto. O usuário descreveu exatamente esse sintoma.

`<br>` passou a serializar como quebra DURA (`"  \n"`): um `\n` sozinho
é quebra suave em markdown e sumiria ao reabrir — a linha quebrada com
Enter voltaria colada na anterior.
