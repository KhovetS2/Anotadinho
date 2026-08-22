---
title: Ciclo 194 — Modos explícitos e teclas por modo
type: ciclo
ciclo: "194"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: [175, 193]
tags:
- ciclo
---

# Ciclo 194 — Modos explícitos e teclas por modo

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

# 194 — Modos explícitos e teclas por modo

## O bug (perda de dado)

Digitar uma sequência com `d` no editor apagava um bloco por letra. Os
atalhos de bloco dependiam só de "existe um bloco focado?", e desde o
ciclo 175 o bloco focado é onde a pessoa digita.

O diagnóstico veio do próprio usuário, e estava certo.

## O que mudou

- `ui/src/components/editor.rs`
  - `Modo` (`Navegacao`/`VimNormal`/`Edicao`) com rótulo, cor, dica e
    lista de atalhos — um modo novo entra num lugar só.
  - Prop `nav_mode_active`; `d`, `y`, `n`, `c`, `K`, `J` só disparam em
    navegação.
  - Indicador de modo na barra de baixo.
  - **Caminho SEM embed também passou a `contenteditable="false"`** — a
    reescrita do 175 só tinha tratado o caminho com embeds.
  - Enter quebra linha (`quebra_de_linha`), Shift+Enter cria bloco, e
    Shift+Enter num `<pre>` sai do código (`bloco_novo_depois`).
- `ui/src/html_to_md.rs`: `<br>` vira quebra DURA; `trim_end` no
  parágrafo pra o `<br>` de bloco vazio não virar lixo no arquivo.
- `ui/src/app.rs`, `page_view.rs`: repasse do modo.
- `ui/src/styles/components.css`: foco sem fundo (só borda), e a dica de
  bloco vazio só na página vazia ou no hover.
- `scripts/uitest/digitacao.mjs`: 2 cenários novos, sendo um do tipo
  "isto NÃO pode acontecer".

## Sobre o travamento durante a validação

O webview congelou numa execução da suíte e eu suspeitei de laço no
código novo. Não era: depois de reiniciar o app, a mesma bateria passou
inteira. Foi o webview enroscado por recargas manuais repetidas somadas
à suíte. Registrado porque a suspeita inicial estava no lugar errado.

## Validação

- `cargo test --workspace`: 0 falhas; `ui`: 39 testes.
- `trunk build`: `✅ success`; Tauri: 0 erros.
- `node scripts/uitest/run.mjs`: **45/45 em 253.9s**.
- Na janela: modo mostra `EDIÇÃO`, Enter mantém um bloco só e
  Shift+Enter cria o segundo.
