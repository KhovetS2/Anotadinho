---
id: "076"
titulo: "Corrige duplicacao de texto na celula Text da tabela embed"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Corrige duplicação de texto na célula Text da tabela embed

## Objetivo

Usuário reportou: um input de texto duplica o texto ao sair de foco ou
clicar fora, e também duplica ao criar uma nova linha. Raiz encontrada na
coluna `Text` da tabela embed (`InlineTable`), a única que usava um `<td
contenteditable="true">` em vez de um `<input>` de verdade — igual todas
as outras colunas (Number, Date, Url, PageLink) já faziam.

## Critérios de aceite

- [x] Coluna `Text` troca `<td contenteditable>` por
      `<input type="text">` dentro do `<td>` — mesmo padrão já usado pela
      coluna `Number`
- [x] Editar uma célula, sair do foco (blur) e conferir que o texto não
      duplica
- [x] Editar uma célula, deixar OUTRA célula disparar um re-render da
      tabela inteira, conferir que não duplica nenhuma das duas

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Outros usos de `contenteditable` + `onblur` no app (título de card do
  kanban, descrição, título de evento do calendário, nome de coluna) —
  têm o MESMO padrão vulnerável em teoria, mas não foram reportados como
  bugados e são campos de título/descrição de uma linha só (sem a
  "criação de nova linha" que dispara o pior caso do bug). Ficam pra um
  ciclo futuro se forem reportados.

## Notas

**Causa raiz**: `<td contenteditable="true">{ cell }</td>` sem
`oninput` — o Yew só sabe o texto que ELE renderizou por último (o
"anterior" cacheado no VDOM), nunca o que o navegador foi acumulando
enquanto o usuário digitava direto no DOM (fora do controle do Yew, já
que não tem handler de `oninput` reagindo a cada tecla). Duas
consequências:

1. No blur, o handler lê `el.text_content()` (texto real, correto) e
   chama `on_change` → o Yew tenta atualizar o nó de texto pro valor
   novo, mas se digitar um Enter no meio criou um `<div>`/quebra de
   linha novo (comportamento padrão de contenteditable no WebKit), esse
   nó extra nunca foi rastreado pelo Yew — nunca é removido ao
   reconciliar, sobra como conteúdo "duplicado" na célula.
2. Mesmo sem Enter, qualquer re-render da tabela inteira (editar OUTRA
   célula, por exemplo) podia mexer na árvore de forma que o Yew
   perdesse a referência exata do nó de texto que o navegador estava
   editando.

**Fix**: trocar por `<input type="text">` — mesmo padrão que a coluna
`Number` já usava (`value={cell.clone()}` + `onblur` lendo
`input.value()`, reaproveitando o helper `input_value()` já existente).
`<input>` não sofre desse problema de jeito nenhum: o valor é uma
propriedade do elemento, não filhos de DOM reconciliados pelo virtual
DOM, e não existe "criar nova linha" nele (Enter não faz nada especial
num `<input type="text">`).

Validado ao vivo via MCP `tauri` na tabela embed de `exemplos-embeds.md`:
editar a célula "Tarefa" da primeira linha (contenteditable → agora
`<input>`) pra "API renovada", disparar `blur`, conferir `input.value ===
"API renovada"` (sem duplicação). Editar OUTRA célula em seguida
(forçando um re-render da tabela inteira) e conferir que a primeira
continua correta. Edições de teste revertidas com `git checkout` antes
de trocar de página (autosave estava ligado, então sem isso teria
vazado pro vault).
