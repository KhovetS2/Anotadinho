---
id: "079"
titulo: "Embeds montam ao vivo sem precisar salvar (corrige insercao corrompida)"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: ["078"]
estima_min: 120
agente_alvo: claude-sonnet
---

# Embeds montam ao vivo sem precisar salvar (corrige inserção corrompida)

## Objetivo

Usuário pediu: embeds deveriam montar o componente de verdade assim que
o conteúdo estiver correto, sem precisar salvar antes. Investigando,
achei que o problema real era mais grave: inserir um embed via slash
command dentro de um item de lista (ou às vezes até num parágrafo com
texto) corrompia a sintaxe do wrapper (`{{ type: "..." }}` virava
`- {{ type: "..." }}` ou tinha texto vazando pro parágrafo vizinho) —
nesses casos o embed NUNCA virava componente de verdade, nem depois de
salvar, porque o parser não reconhecia mais a abertura do wrapper.

## Critérios de aceite

- [x] Inserir Kanban/Calendário/Tabela com o cursor dentro de um item de
      lista vazio monta o componente de verdade IMEDIATAMENTE, sem
      precisar salvar
- [x] Mesmo teste com o cursor no fim de um parágrafo com texto
- [x] Sem duplicação de conteúdo visível na transição de "sem embed" pra
      "com embed" (bug adicional achado durante a validação)
- [x] Outros itens do menu slash (headings, listas, etc, que não passam
      por essa lógica) continuam funcionando sem regressão

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Redesenhar a inserção de TODOS os itens do menu slash (imagem,
  diagrama, tabela markdown simples etc) pra usar `Range` em vez de
  `execCommand` — só os 3 itens de embed (kanban/calendário/tabela
  tipada) foram trocados, que são os únicos com o requisito de "linha
  própria" que o `execCommand` quebrava

## Notas

**Três bugs distintos, achados em sequência durante a investigação** (a
causa inicial suspeitada — "só falta recalcular sem esperar o save" —
não era o problema real):

1. **`execCommand("insertHTML", ...)` fragmenta HTML multi-linha de
   forma imprevisível no WebKitGTK**: dependendo de onde o cursor
   estava, o texto do corpo do embed vazava pro parágrafo vizinho com
   estilo inline estranho, ou a abertura do wrapper saía grudada com o
   marcador de lista. Trocado por inserção via `Range::insert_node`
   (`insert_embed_marker_at_cursor`), que insere o nó exatamente onde o
   cursor está sem o `execCommand` reinterpretar o HTML ao redor.

2. **Mesmo com `Range`, um embed dentro de um `<li>`/`<p>`/blockquote/
   heading ainda quebrava**: `html_to_markdown` converte esses elementos
   com `inline_children()`, que prefixa/envolve o conteúdo (ex: `<li>`
   vira `- {conteúdo}`) — um marcador de embed aninhado ali vira
   `- {{ type: "kanban" }}`, e a abertura do wrapper deixa de estar
   sozinha na linha (exigência do parser). Corrigido detectando o
   ancestral de bloco mais próximo (`closest("li, p, blockquote, h1..h6")`)
   e inserindo o marcador como IRMÃO desse bloco (ou da lista inteira, no
   caso de `<li>`) em vez de aninhado dentro dele.

3. **Duplicação visual na transição sem-embed → com-embed**: o Yew reusa
   o mesmo `<div>` físico ao trocar do branch "sem embeds" (single
   `contenteditable`, conteúdo injetado via `set_inner_html`, fora do
   rastreamento do VDOM) pro branch "com embeds" (`editor__wysiwyg-segments`),
   já que os dois renderizam `<div>` como raiz — sem uma identidade
   explícita, o Yew só ajusta a classe do nó existente e ANEXA os filhos
   novos, sem limpar o conteúdo antigo (nunca rastreado por ele).
   Corrigido com `key="segments"`/`key="plain"` nos dois branches,
   forçando desmontagem/remontagem completa.

Junto com essa mudança de mecanismo de inserção, o slash command pra
embed agora recalcula e aplica o markdown NA HORA (via `mark_edited`,
extraído no ciclo 078), então o componente de verdade aparece
imediatamente — resolvendo o pedido original também, não só o bug de
corrupção.

Validado ao vivo via MCP `tauri`: inserir Kanban com o cursor num item de
lista vazio (`<li></li>`) — board interativo aparece na hora, sem
duplicação, sem precisar clicar "Salvar". Repetido pros 3 tipos de embed
(kanban/calendário/tabela). "Salvar" manual depois confirma o arquivo no
disco bem formado (`{{ type: "kanban" }}` sozinho na linha). Item de
menu NÃO relacionado a embed (Título 1) testado sem regressão, continua
usando `execCommand` normalmente.

Todas as edições de teste foram revertidas com `git checkout` antes de
prosseguir — nenhuma vazou pro vault de verdade.
