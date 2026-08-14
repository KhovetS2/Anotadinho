---
id: "143"
titulo: "Checkbox via [ ]/[] e combinação lista+checkbox"
status: done
criado: 2026-08-14
autor: humano
prioridade: alta
depende_de: ["142"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Checkbox via [ ]/[] e combinação lista+checkbox

## Objetivo

Dois bugs reais reportados pelo usuário: digitar `"[ ]"` ou `"[]"` não
cria um checkbox, e não dá pra combinar lista + checkbox manualmente
(só via agente/CLI escrevendo o markdown direto). Quatro causas raiz
distintas, todas na mesma área:

1. Não existe NENHUM atalho de digitação pra checkbox — só o item
   "Checklist" do menu `/`, que insere `<input>` solto num `<div>`
   (não um `<li>` de verdade, então não é uma "lista+checkbox").
2. `html_to_md.rs::walk`, ramo `"li"`: sempre prefixa `"- "` (ou
   `"1. "`) na frente do conteúdo, mesmo quando o conteúdo já É um
   checkbox (que o ramo `"input"` já serializa como `"- [ ] "`) —
   resultado: `"- - [ ] texto"`, marcador duplicado, markdown quebrado.
3. Ramo `"input"` do `walk` sempre devolve `"- [ ] "` (desmarcado),
   nunca lê o estado `checked` de verdade do checkbox.
4. Clicar num checkbox pra marcar/desmarcar nunca é persistido —
   `on_wysiwyg_click` só trata clique em link wikilink; o clique nativo
   no `<input>` só dispara `"click"`, não `"input"`, então
   `mark_edited` nunca roda pra esse caso.

## Critérios de aceite

- [x] `apply_block_shortcut` (editor.rs): novo atalho — prefixo `"[]"`
      ou `"[ ]"` + espaço insere um `<input type="checkbox">` no
      cursor (mesmo padrão de `select_prefix` + `exec_cmd("delete")`
      já usado pelos outros atalhos do ciclo 142), funcionando tanto
      solto num parágrafo quanto dentro de um item de lista já
      convertido pelo atalho `"- "` (o combo pedido pelo usuário)
- [x] `html_to_md.rs::walk`, ramo `"li"`: se o conteúdo já começar com
      `"- [ ] "`/`"- [x] "` (veio do ramo `"input"`), NÃO prefixa outro
      marcador — só usa o que já veio
- [x] `html_to_md.rs::walk`, ramo `"input"`: lê o estado `checked` de
      verdade (`HtmlInputElement::checked()`, não o atributo
      `checked` — que não reflete toggle feito pelo usuário) e emite
      `"- [x] "` quando marcado
- [x] `on_wysiwyg_click` (editor.rs): clique em `input[type=checkbox]`
      dentro do editor recalcula o markdown do DOM e chama
      `mark_edited` — sem isso, marcar/desmarcar nunca persiste
- [x] Template do item "Checklist" do menu `/` muda de
      `<div><input type='checkbox'> Tarefa</div>` pra
      `<ul><li><input type='checkbox'> Tarefa</li></ul>` — mesma forma
      que o round-trip real produz (consistente com o item "Lista", que
      já usa `<ul><li>`), evitando que a aparência mude entre "acabou
      de inserir" e "salvou e recarregou"
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `cd ui &&
      trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: lista com um item normal +
      um item `"[] "` (combo lista+checkbox); marcar o checkbox e
      salvar — arquivo no disco ficou `"- [x] tarefa combinada"`,
      sem marcador duplicado e com o estado marcado preservado

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Suporte a `"[x]"` digitado pra já nascer marcado — não foi pedido,
  fora de escopo deste ciclo
- Reordenar/indentar itens de checklist por teclado — cai no nav-mode/
  teclado geral de listas, que já funciona via `execCommand` nativo do
  contenteditable, não precisa de código novo aqui

## Notas

`cd ui && cargo test --lib`: 84 passados. `cargo test --workspace`:
OK. `trunk build`/`cargo build --manifest-path src-tauri/Cargo.toml`:
OK.

Validação ao vivo via MCP `tauri` (sequência completa, na mesma
página): criei `<ul><li>primeiro item</li></ul>` com o atalho `"-"` +
espaço (ciclo 142), depois um segundo `<li>` vazio, digitei `"[]"`
nele e mandei um evento de tecla espaço de verdade — virou
`<input type="checkbox">` dentro do MESMO item de lista (combo lista+
checkbox pedido pelo usuário). Digitei o texto da tarefa, cliquei no
checkbox pra marcar (`checked` virou `true`), salvei — o arquivo no
disco ficou:
```
- primeiro item
- [x] tarefa combinada
```
Sem marcador duplicado (`"- - [x]"`, o bug antigo) e com o estado
marcado preservado corretamente.

Nota sobre o ambiente de teste: assim como no ciclo 142, `Enter`
sintético via automação não aciona o comportamento nativo de
continuar uma lista (só eventos confiáveis fazem isso) — contornei
criando o segundo `<li>` diretamente via DOM/Range antes de disparar
o atalho de teclado de verdade, pra validar só a lógica que este ciclo
mudou.
