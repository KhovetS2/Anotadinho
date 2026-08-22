---
title: Ciclo 075 — Botao de hover para adicionar linha acima ou abaixo de um embed
type: ciclo
ciclo: "075"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 075 — Botao de hover para adicionar linha acima ou abaixo de um embed

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Botão de hover pra adicionar linha acima/abaixo de um embed

## Objetivo

Quando um embed nasce sem uma linha de markdown vizinha — é o
primeiro/último segmento da página, ou está colado a outro embed sem
nada entre eles — não tinha nenhum lugar clicável pra digitar texto
naquela borda. Adiciona um botão "+" que só aparece no hover da borda de
cima/baixo do embed, que insere uma linha nova ali e já foca nela.

## Critérios de aceite

- [x] `.embed-hover-wrapper` envolve cada `InlineEmbed`, com botões
      `--top`/`--bottom` revelados só no `:hover` (CSS puro, sem estado
      extra em Rust)
- [x] Clicar insere um segmento de markdown novo na posição certa e foca
      nele automaticamente
- [x] Funciona no caso mais importante: dois embeds colados, sem nenhuma
      linha de markdown entre eles

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhuma mudança no comportamento de embeds que já têm markdown vizinho
  (o botão ainda aparece ali, mas só acrescenta uma linha em branco a
  mais — não é o cenário que motivou o pedido)

## Notas

**Dois erros de implementação encontrados e corrigidos durante a
validação ao vivo** (nenhum dos dois apareceu no primeiro teste, que por
acaso usou um embed com markdown vizinho — só apareceram testando o
cenário real, dois embeds colados):

1. Yew não aceita `let` solto dentro de um branch `if cond { ... }` de um
   `html!` já aberto sem um `html! {}` aninhado explícito — o fechamento
   de chaves ficou errado na primeira tentativa. Corrigido movendo a
   definição do closure `insert_blank_line` pro escopo de nível superior
   da função (antes do `html!` principal), igual a `on_edit`/`save_label`.

2. **Bug real, não só erro de sintaxe**: `embed::join()` não escreve
   NADA (nem quebra de linha) pra um `DocSegment::Markdown("")` vazio —
   então inserir uma linha em branco entre dois embeds colados
   desaparecia de novo assim que `content_md` era serializado e
   reparseado (`segment()` só cria um segmento de markdown se sobrar
   pelo menos 1 caractere entre dois delimitadores de embed — string
   vazia não conta). Corrigido inserindo `Markdown("\n")` em vez de
   `Markdown("")`.

Validado ao vivo via MCP `tauri`: escrevi uma página de teste com dois
kanbans colados direto via IPC (`write_page`, contornando o editor pra
garantir o cenário exato sem markdown nenhum entre eles), cliquei no "+"
de baixo do primeiro embed — apareceu um `<div class="editor__wysiwyg"
data-segment-index="1">` novo entre os dois, com foco automático nele
(confirmado por `document.activeElement`), e digitar nele funcionou
normalmente. Também testado (primeira tentativa, antes de perceber que
não exercitava o cenário real) num embed com markdown já vizinho — nesse
caso só acrescenta uma linha em branco a mais no segmento existente
(comportamento aceitável, não é o caso que motivou o pedido).

Duas edições de teste vazaram pro vault durante a validação (confirmado
via notificação de mudança de arquivo — o flush de segurança do ciclo
074 salvou automaticamente ao trocar de página) e foram revertidas com
`git checkout` depois de confirmadas.

## Resultado

# Ciclo 075 - done

## Resumo

Botão "+" que aparece no hover da borda de cima/baixo de um embed,
insere uma linha de markdown nova ali e foca nela — resolve o caso de um
embed nascer sem nenhuma linha vizinha clicável (primeiro/último
segmento, ou colado a outro embed).

## Arquivos criados/modificados

- `ui/src/components/editor.rs` — closure `insert_blank_line`,
  `data-segment-index` nos segmentos de markdown, `.embed-hover-wrapper`
  ao redor de cada `InlineEmbed` com os dois botões
- `ui/src/styles/main.css` — `.embed-hover-wrapper*`

## Testes

`cargo test --lib`: 52 passaram (sem testes novos — depende de DOM real,
sem harness de wasm-bindgen-test; validado via MCP ao vivo).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Dois bugs de implementação achados e corrigidos durante a validação
(detalhes na task): sintaxe de `let` dentro de branch `if` do `html!`
sem wrapper explícito, e — mais importante — `embed::join()` descarta
`Markdown("")` silenciosamente (zero bytes escritos), fazendo a linha
inserida sumir de novo ao reparsear; corrigido inserindo `Markdown("\n")`.

Teste real (dois embeds colados, escrito via IPC direto pra garantir o
cenário exato): clicar no "+" criou um segmento novo com foco automático,
confirmado editável.

## Notas

Duas edições de teste vazaram pro vault via o flush de segurança do
ciclo 074 (esperado — é exatamente pra isso que ele existe) e foram
revertidas com `git checkout` depois de confirmadas.
