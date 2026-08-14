---
id: "141"
titulo: "Corrige aninhamento duplo de fence de código no round-trip HTML→Markdown"
status: done
criado: 2026-08-14
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Corrige aninhamento duplo de fence de código no round-trip HTML→Markdown

## Objetivo

Bug real reportado pelo usuário: qualquer bloco de código na página faz
`Ctrl+Z` "quebrar todas as formatações", o editor "quebrar ao inserir
formatações via markdown" e a "formatação de código quebrar quando é
salvo e depois reabre a página". Causa raiz: `html_to_md.rs::walk`, no
ramo `"pre"`, chama `text_of(node)` — que desce (via
`inline_children`) até o `<code>` filho e roda o ramo `"code"` do
próprio `walk`, que (ao ver que o pai é `<pre>`) já devolve o texto
RE-ENVOLTO em ` ``` `. O `"pre"` então envolve esse resultado em outra
fence por cima, produzindo fences aninhadas quebradas
(` ```\n```\ncódigo\n```\n\n```\n\n `) todo santo save. Como
`recompute_markdown_from_dom` (chamado a cada `oninput`, ou seja, a
cada tecla digitada em QUALQUER lugar da página) roda esse caminho
sobre o DOM inteiro, qualquer página com bloco de código se corrompe
já na primeira edição depois de carregar — e como o histórico de
undo/redo empilha esses markdowns já corrompidos, `Ctrl+Z` "restaura"
uma versão igualmente quebrada.

## Critérios de aceite

- [x] `html_to_md.rs::walk`, ramo `"pre"`: para de chamar `text_of`
      (que reentra no ramo `"code"`) — extrai o texto cru direto do
      `<code>` filho via `query_selector("code")` +
      `text_content()`, sem passar pelo `walk`/`"code"` de novo
- [x] Ramo `"code"` do `walk` simplificado: já não precisa mais checar
      `parent_tag == "pre"` (esse caminho nunca mais é alcançado a
      partir de um `<pre>`, já que `"pre"` não desce mais nele) —
      remove o branch morto, deixa só o caso de código inline
      (`` `x` ``)
- [x] Validação ao vivo via MCP (ver Notas) fez as vezes do teste
      automatizado — `html_to_markdown` depende de `web_sys::Element`,
      não é testável em Rust puro sem um DOM real
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `trunk
      build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: criar página com bloco de
      código (` ```rust\nfn main() {}\n``` `), digitar QUALQUER outro
      caractere em outro parágrafo da mesma página (forçando um
      `oninput`/recompute), salvar, fechar a página e reabrir —
      confirmar que o bloco de código continua exatamente igual (sem
      fences aninhadas); repetir o teste incluindo um `Ctrl+Z` no meio

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Reescrever o parser HTML→Markdown pra uma abordagem baseada em AST
  (fora de escopo — o bug é um caso específico e pontual de
  reentrância acidental entre dois ramos do `walk`)
- Preservar syntax highlighting/classes do highlight.js no texto do
  código — `text_content()` já ignora `<span>` de highlight e retorna
  só o texto puro, que é o comportamento correto pra markdown

## Notas

`cd ui && cargo test --lib`: 84 passados. `cargo test --workspace`:
todas as sub-crates OK. `trunk build` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: criei uma página com um parágrafo,
um bloco ` ```rust ` e outro parágrafo depois. Digitei um caractere
solto no parágrafo final (forçando `recompute_markdown_from_dom` sobre
o DOM inteiro, incluindo o bloco de código já renderizado com
highlight.js) e salvei — o arquivo no disco manteve o bloco de código
intacto, com uma única fence (sem aninhamento). Antes do fix, esse
mesmo passo produzia fences duplicadas/aninhadas. Testei também
`Ctrl+Z` logo depois do save: desfez a inserção do caractere sem
quebrar a formatação do bloco de código (que continuou renderizando
com highlight normalmente), confirmando que o bug relatado como
"Ctrl+Z quebra todas as formatações" também estava raiz nesse mesmo
problema — o histórico de undo só "restaurava" markdown já corrompido
pelo bug do round-trip, não era um defeito na lógica de undo em si.
