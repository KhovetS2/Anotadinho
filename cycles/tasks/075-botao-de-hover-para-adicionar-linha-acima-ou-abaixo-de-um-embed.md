---
id: "075"
titulo: "Botao de hover para adicionar linha acima ou abaixo de um embed"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
